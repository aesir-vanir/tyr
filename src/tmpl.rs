//! `tyr` output templates
use error::Result;
use inflector::cases::snakecase::to_snake_case;
use inflector::cases::pascalcase::to_pascal_case;
use mustache;
use run::Rows;
use std::collections::BTreeMap;
use std::io::{self, Cursor, Write};

/// Table struct mustache template.
const ORM_TMPL: &'static str = "//! ORM generated by tyr
use error::Result;
use mimir::Connection;

{{#tables}}/// `{{struct_name}}` ORM
#[derive(Getters, MutGetters, Setters, {{#derives}}{{name}}{{#comma}}, {{/comma}}{{/derives}})]
pub struct {{struct_name}} {
    {{#field}}
    /// `{{field_name}}` column
    #[get = \"pub\"]
    #[set = \"pub\"]
    #[get_mut = \"pub\"]
    {{field_name}}: {{{field_type}}},
    {{/field}}
}

impl {{struct_name}} {
    /// Fetch a vector of `{{struct_name}}` from the given connection.  By default, all rows will be
    /// fetched.
    pub fn fetch(conn: &Connection) -> Result<Vec<{{struct_name}}>> {
        Ok(Vec::new())
    }
}
{{/tables}}";

/// `File` information used for mustache template.
#[derive(Builder, Clone, Debug, Default, Deserialize, RustcEncodable, Eq, Hash, PartialEq, Serialize, Setters)]
struct File {
    /// Tables to include in this template.
    #[set]
    tables: Vec<Table>,
}

/// `Derive` information used for mustache template.
#[derive(Builder, Clone, Debug, Default, Deserialize, RustcEncodable, Eq, Getters, Hash, PartialEq, Serialize)]
struct Derive {
    /// The derive name.
    name: String,
    /// Include a trailing comma?
    comma: bool,
}

/// `Table` information used for mustache template.
#[derive(Builder, Clone, Debug, Default, Deserialize, RustcEncodable, Eq, Getters, Hash, PartialEq, Serialize)]
struct Table {
    /// The list of derives.
    derives: Vec<Derive>,
    /// The struct name tag.
    // #[get]
    struct_name: String,
    /// The struct fields.
    // #[get]
    field: Vec<Field>,
}

/// `Field` information used for mustache template.
#[derive(Clone, Debug, Default, Deserialize, Eq, Getters, Hash, PartialEq, RustcEncodable, Serialize, Setters)]
struct Field {
    /// The field name tag.
    #[set]
    field_name: String,
    /// The field type tag.
    #[set]
    #[get]
    field_type: String,
    /// Is this field nullable?
    #[set]
    nullable: bool,
}

/// Render a table from the mustache template.
pub fn render(table_info: &BTreeMap<String, Rows>) -> Result<()> {
    let template = mustache::compile_str(ORM_TMPL)?;
    let mut file: File = Default::default();
    let mut tables: Vec<Table> = Vec::new();

    for (table_name, rows) in table_info {
        let mut fields: Vec<Field> = Vec::new();

        for col_info in rows.values() {
            let mut field: Field = Default::default();

            for col in col_info {
                match &(*col.column_name())[..] {
                    "COLUMN_NAME" => {
                        let type_info = col.type_info();
                        let data = if let Some(ref data) = *col.data() {
                            data.to_string(type_info)?
                        } else {
                            "(null)".to_string()
                        };
                        field.set_field_name(to_snake_case(&data));
                    }
                    "DATA_TYPE" => {
                        let type_info = col.type_info();
                        let data = if let Some(ref data) = *col.data() {
                            data.to_string(type_info)?
                        } else {
                            "(null)".to_string()
                        };
                        let mapped = match &data[..] {
                            "NUMBER" => "f64",
                            "VARCHAR2" => "String",
                            _ => "",
                        };
                        field.set_field_type(mapped.to_string());
                    }
                    "NULLABLE" => {
                        let type_info = col.type_info();
                        let data = if let Some(ref data) = *col.data() {
                            data.to_string(type_info)?
                        } else {
                            "(null)".to_string()
                        };
                        match &data[..] {
                            "Y" => {
                                let mut optional = String::from("Option<");
                                optional.push_str(field.field_type());
                                optional.push_str(">");
                                field.set_field_type(optional);
                                field.set_nullable(true)
                            }
                            "N" => field.set_nullable(false),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            fields.push(field);
        }

        let mut derives = Vec::new();
        let derive_names = vec!["Clone", "Default", "Debug", "Eq", "Hash", "PartialEq"];
        for (idx, derive) in derive_names.iter().enumerate() {
            let derive: Derive = DeriveBuilder::default()
                .name(derive.to_string())
                .comma(idx < (derive_names.len() - 1))
                .build()?;
            derives.push(derive);
        }

        let table: Table = TableBuilder::default()
            .struct_name(to_pascal_case(table_name))
            .derives(derives)
            .field(fields)
            .build()?;

        tables.push(table);
    }
    file.set_tables(tables);
    let mut out = Cursor::new(Vec::new());
    template.render(&mut out, &file)?;
    writeln!(io::stdout(), "{}", String::from_utf8(out.into_inner())?)?;
    Ok(())
}
