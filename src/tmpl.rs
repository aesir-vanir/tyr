//! `tyr` output templates
use error::Result;
use inflector::cases::snakecase::to_snake_case;
use inflector::cases::pascalcase::to_pascal_case;
use mustache;
use run::Rows;
use std::collections::BTreeMap;
use std::io::{self, Cursor, Write};

/// Table struct mustache template.
const TABLE_STRUCT: &'static str = r"/// {{struct_name }}
#[derive(Debug, Default)]
pub struct {{struct_name}} {
    {{#field}}
    {{field_name}}: {{field_type}},
    {{/field}}
}";

/// `Table` information used for mustache template.
#[derive(Builder, Clone, Debug, Default, Deserialize, RustcEncodable, Eq, Getters, Hash, PartialEq, Serialize)]
pub struct Table {
    /// The struct name tag.
    // #[get]
    struct_name: String,
    /// The struct fields.
    // #[get]
    field: Vec<Field>,
}

/// `Field` information used for mustache template.
#[derive(Clone, Debug, Default, Deserialize, Eq, Getters, Hash, PartialEq, RustcEncodable, Serialize, Setters)]
pub struct Field {
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
pub fn render(tables: &BTreeMap<String, Rows>) -> Result<()> {
    let template = mustache::compile_str(TABLE_STRUCT)?;

    for (table_name, rows) in tables {
        let mut fields: Vec<Field> = Vec::new();

        for (_row_idx, col_info) in rows {
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
                        let mut col_name = String::from(table_name.clone());
                        col_name.push('_');
                        col_name.push_str(&data);
                        field.set_field_name(to_snake_case(&col_name));
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
                                optional.push_str(&field.field_type());
                                optional.push_str(">");
                                field.set_field_type(optional);
                                field.set_nullable(true)
                            },
                            "N" => field.set_nullable(false),
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            fields.push(field);
        }


        let table: Table = TableBuilder::default()
            .struct_name(to_pascal_case(table_name))
            .field(fields)
            .build()?;

        let mut out = Cursor::new(Vec::new());
        template.render(&mut out, &table)?;
        writeln!(io::stdout(), "{}", String::from_utf8(out.into_inner())?)?;
    }
    Ok(())
}
