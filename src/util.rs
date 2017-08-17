//! `tyr` utilities
use error::{ErrorKind, Result};
use run::QueryDataByCol;
use std::collections::HashMap;
use term;

/// Pad a string to the given length.
fn pad_to_len(len: usize, s: &str) -> String {
    let mut output = String::from(s);
    let s_len = s.len();
    if s_len < len {
        let pad_len = len - s_len;
        for _ in 0..pad_len {
            output.push(' ');
        }
    }

    output
}

/// Pretty print table column information.
pub fn pretty_print_tables(tables: &HashMap<String, Vec<QueryDataByCol>>) -> Result<()> {
    for (table, qibc_vec) in tables {
        let mut t = term::stdout().ok_or_else(|| ErrorKind::Stdout)?;
        t.attr(term::Attr::Bold)?;
        t.fg(term::color::GREEN)?;
        writeln!(t, "Table '{}'", table)?;
        t.reset()?;
        t.flush()?;

        t.fg(term::color::YELLOW)?;
        t.attr(term::Attr::Bold)?;
        write!(t, "  ")?;
        for (idx, qibc) in qibc_vec.iter().enumerate() {
            write!(t, "{}", pad_to_len(*qibc.max_length(), qibc.column_name()))?;
            if idx < qibc_vec.len() {
                write!(t, "  ")?;
            }
        }
        writeln!(t, "")?;
        t.reset()?;
        t.flush()?;

        let row_count = qibc_vec[0].row_count();

        t.fg(term::color::GREEN)?;
        for i in 0..row_count {
            write!(t, "  ")?;
            for (idx, qibc) in qibc_vec.iter().enumerate() {
                let data_vec = qibc.data();
                let data_type = qibc.type_info();
                write!(
                    t,
                    "{}",
                    pad_to_len(*qibc.max_length(), &data_vec[i].to_string(data_type)?)
                )?;
                if idx < qibc_vec.len() {
                    write!(t, "  ")?;
                }
            }
            writeln!(t, "")?;
        }
        t.reset()?;
        t.flush()?;
    }

    Ok(())
}
