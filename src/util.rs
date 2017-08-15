//! `tyr` utilities
use error::{ErrorKind, Result};
use run::ColumnInfo;
use std::collections::HashMap;
use term;

/// Pretty print table column information.
pub fn pretty_print_tables(tables: &HashMap<String, Vec<ColumnInfo>>) -> Result<()> {
    for (table, col_info_vec) in tables {
        let mut t = term::stdout().ok_or_else(|| ErrorKind::Stdout)?;
        t.attr(term::Attr::Bold)?;
        t.fg(term::color::GREEN)?;
        writeln!(t, "Table '{}'", table)?;
        t.reset()?;
        t.flush()?;
        let mut max_name_len = 0;
        let mut max_nullable_len = 0;
        let mut col_info_strs = Vec::new();

        for col_info in col_info_vec {
            col_info_strs.push(col_info.to_string());
            let name_len = col_info.name().len();
            let nullable_len = if let Some(val) = *col_info.nullable() {
                val.to_string().len()
            } else {
                "(null)".len()
            };

            if name_len > max_name_len {
                max_name_len = name_len;
            }

            if nullable_len > max_nullable_len {
                max_nullable_len = nullable_len;
            }
        }

        let mut name = String::from("  Name");
        let name_len = name.len();

        if name_len > max_name_len {
            max_name_len = name_len;
        } else if name_len < max_name_len {
            let total = (max_name_len - name_len) + 2;
            for _ in 0..total {
                name.push(' ');
            }
        }

        let mut nullable = String::from("Nullable");
        let nullable_len = nullable.len();

        if nullable_len > max_nullable_len {
            max_nullable_len = nullable_len;
        } else if nullable_len < max_nullable_len {
            let total = (max_nullable_len - nullable_len) + 2;
            for _ in 0..total {
                nullable.push(' ');
            }
        }

        t.fg(term::color::YELLOW)?;
        t.attr(term::Attr::Bold)?;
        writeln!(t, "{}  {}  {}", name, nullable, "Data Type")?;
        t.reset()?;
        t.flush()?;
        t.fg(term::color::GREEN)?;
        for col_info in col_info_vec {
            let mut col_name = col_info.name().clone();
            let col_name_len = col_name.len();
            if col_name_len < max_name_len {
                let total = max_name_len - col_name_len;
                for _ in 0..total {
                    col_name.push(' ');
                }
            }

            let mut nullable_str = if let Some(val) = *col_info.nullable() {
                val.to_string()
            } else {
                "(null)".to_string()
            };
            let nullable_str_len = nullable_str.len();
            if nullable_str_len < max_name_len {
                let total = max_nullable_len - nullable_str_len;
                for _ in 0..total {
                    nullable_str.push(' ');
                }
            }

            let data_type_str = if let Some(ref val) = *col_info.data_type() {
                val.clone()
            } else {
                "(null)".to_string()
            };
            writeln!(
                t,
                "  {}  {}  {}({})",
                col_name,
                nullable_str,
                data_type_str,
                col_info.data_length()
            )?;
        }
        t.reset()?;
        t.flush()?;
        for col_info_str in col_info_strs {
            writeln!(t, "{}", col_info_str)?;
        }
    }

    Ok(())
}
