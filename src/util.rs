//! `tyr` utilities
use error::{ErrorKind, Result};
use run::ColumnInfo;
use std::collections::HashMap;
use term;

/// Column headers
const COLS: &'static str = "Name|Data Type|Data Type Mod|Data Type Owner|Data Length|\
                            Data Precision|Data Scale|Nullable|Column ID|Default Length|Distinct|\
                            Low Value|Last Analyzed";
/// Number of printed columns.
const COL_COUNT: usize = 13;

/// Pretty print table column information.
pub fn pretty_print_tables(tables: &HashMap<String, Vec<ColumnInfo>>) -> Result<()> {
    for (table, col_info_vec) in tables {
        let mut t = term::stdout().ok_or_else(|| ErrorKind::Stdout)?;
        t.attr(term::Attr::Bold)?;
        t.fg(term::color::GREEN)?;
        writeln!(t, "Table '{}'", table)?;
        t.reset()?;
        t.flush()?;

        let mut col_info_strs = Vec::new();

        col_info_strs.push(COLS.to_string());

        for col_info in col_info_vec {
            col_info_strs.push(col_info.to_string());
        }

        let mut max_col_lens = vec![0; COL_COUNT];
        for col_info_str in &col_info_strs {
            let split: Vec<&str> = col_info_str.split('|').collect();
            for (i, x) in split.iter().enumerate() {
                let curr_len = x.len();
                if curr_len > max_col_lens[i] {
                    max_col_lens[i] = curr_len;
                }
            }
        }

        for (idx, col_info_str) in col_info_strs.iter().enumerate() {
            let split: Vec<&str> = col_info_str.split('|').collect();
            write!(t, "  ")?;
            for (len, d) in max_col_lens.iter().zip(split.iter()) {
                let d_len = d.len();
                let padded = if d_len < *len {
                    let pad_len = *len - d_len;
                    let mut padded = String::from(*d);
                    for _ in 0..pad_len {
                        padded.push(' ');
                    }
                    padded
                } else {
                    d.to_string()
                };

                if idx == 0 {
                    t.fg(term::color::YELLOW)?;
                    t.attr(term::Attr::Bold)?;
                } else {
                    t.fg(term::color::GREEN)?;
                }

                write!(t, "{}  ", padded)?;
                t.reset()?;
                t.flush()?;
            }
            writeln!(t, "")?;
        }
    }

    Ok(())
}
