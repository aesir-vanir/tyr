// Copyright (c) 2017 tyr developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `tyr` runtime
use clap::{App, Arg};
use context::{Context, ContextBuilder};
use error::{ErrorKind, Result};
use mimir::enums::ODPINativeTypeNum::Bytes;
use mimir::enums::ODPIOracleTypeNum::Varchar;
use mimir::{flags, Connection, Data, TypeInfo};
use std::collections::HashMap;
use std::ffi::CString;
use std::fmt;
use util;

/// User space table names query.
const TABLE_NAMES: &'static str = r"select table_name from user_tables";
/// Describe user space tables Oracle SQL.
const DESC: &'static str = r"SELECT TABLE_NAME, COLUMN_NAME, DATA_TYPE, DATA_TYPE_MOD,
DATA_TYPE_OWNER, DATA_LENGTH, DATA_PRECISION, DATA_SCALE, NULLABLE, COLUMN_ID, DEFAULT_LENGTH,
NUM_DISTINCT, LOW_VALUE, HIGH_VALUE, DENSITY, NUM_NULLS, NUM_BUCKETS, LAST_ANALYZED, SAMPLE_SIZE,
CHARACTER_SET_NAME, CHAR_COL_DECL_LENGTH, GLOBAL_STATS, USER_STATS, AVG_COL_LEN, CHAR_LENGTH,
CHAR_USED, V80_FMT_IMAGE, DATA_UPGRADED, HISTOGRAM, DEFAULT_ON_NULL, IDENTITY_COLUMN,
EVALUATION_EDITION, UNUSABLE_BEFORE, UNUSABLE_BEGINNING
FROM user_tab_columns
WHERE table_name=:table_name";

/// Query Data by Column
#[derive(Debug, Default, Getters, MutGetters, Setters)]
pub struct QueryDataByCol {
    /// The column name.
    #[get = "pub"]
    #[set]
    column_name: String,
    /// The maximum length (as a `String`) in the column.
    #[get = "pub"]
    #[get_mut]
    #[set]
    max_length: usize,
    /// The column native data type.
    #[get = "pub"]
    #[set]
    type_info: TypeInfo,
    /// The column data.
    #[get = "pub"]
    #[get_mut]
    data: Vec<Data>,
}

impl QueryDataByCol {
    /// Get the row count for this col data.
    pub fn row_count(&self) -> usize {
        self.data.len()
    }
}

impl fmt::Display for QueryDataByCol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.column_name,
            self.max_length,
            self.data.len()
        )
    }
}

/// Connect to the database.
fn conn(ctxt: &Context) -> Result<()> {
    let db_ctxt = ctxt.db_context();
    let mut common_create_params = db_ctxt.init_common_create_params()?;
    let enc_cstr = CString::new("UTF-8").expect("badness");
    common_create_params.set_encoding(enc_cstr.as_ptr());
    common_create_params.set_nchar_encoding(enc_cstr.as_ptr());
    common_create_params.set_create_mode(flags::DPI_MODE_CREATE_EVENTS);
    let conn = Connection::create(
        db_ctxt,
        Some(ctxt.username()),
        Some(ctxt.password()),
        Some(ctxt.conn_string()),
        Some(common_create_params),
        None,
    )?;

    let user_tables = conn.prepare_stmt(Some(TABLE_NAMES), None, false)?;

    let _ = user_tables.execute(flags::DPI_MODE_EXEC_DEFAULT)?;
    let (mut found, _) = user_tables.fetch()?;
    let mut table_names: HashMap<String, Vec<QueryDataByCol>> = HashMap::new();

    while found {
        let (_id_type, id_ptr) = user_tables.get_query_value(1)?;
        let data: Data = id_ptr.into();
        table_names.insert(data.get_string(), Default::default());
        let (f, _) = user_tables.fetch()?;
        found = f;
    }

    for (table, qdbc_vec) in &mut table_names {
        let table_desc = conn.prepare_stmt(Some(DESC), None, false)?;
        let table_name_var = conn.new_var(Varchar, Bytes, 1, 256, false, false)?;
        table_name_var.set_from_bytes(0, table)?;
        table_desc.bind_by_name(":table_name", &table_name_var)?;

        let cols = table_desc.execute(flags::DPI_MODE_EXEC_DEFAULT)?;
        let (mut found, _buffer_row_index) = table_desc.fetch()?;

        if found {
            for i in 1..(cols + 1) {
                let mut query_data_by_col: QueryDataByCol = Default::default();
                let query_info = table_desc.get_query_info(i)?;
                query_data_by_col.set_column_name(query_info.name());
                query_data_by_col.set_max_length(query_info.name().len());
                query_data_by_col.set_type_info(query_info.type_info());
                qdbc_vec.push(query_data_by_col);
            }
        }

        while found {
            for i in 1..(cols + 1) {
                let idx = (i - 1) as usize;
                let (_, data) = table_desc.get_query_value(i)?;
                let query_data_by_col = &mut qdbc_vec[idx];
                let type_info = *query_data_by_col.type_info();
                let cur_len = *query_data_by_col.max_length();
                let data_len = data.len(&type_info)?;
                if data_len > cur_len {
                    *query_data_by_col.max_length_mut() = data_len;
                }
                (*query_data_by_col.data_mut()).push(data);
            }

            let (f, _) = table_desc.fetch()?;
            found = f;
        }

        table_name_var.release()?;
        table_desc.close(None)?;
    }

    user_tables.close(None)?;
    util::pretty_print_tables(&table_names)?;

    Ok(())
}

/// CLI Runtime
pub fn run() -> Result<i32> {
    use std::io::{self, Write};
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Rust ORM codegen for Oracle.")
        .arg(
            Arg::with_name("conn_string")
                .short("c")
                .long("conn_string")
                .takes_value(true)
                .required(true)
                .value_name("CONN_STRING"),
        )
        .arg(
            Arg::with_name("username")
                .short("u")
                .long("username")
                .takes_value(true)
                .value_name("USERNAME"),
        )
        .arg(
            Arg::with_name("password")
                .short("p")
                .long("password")
                .takes_value(true)
                .value_name("PASSWORD"),
        )
        .get_matches();

    let conn_string = matches
        .value_of("conn_string")
        .ok_or(ErrorKind::ConnectionString)?;
    let username = matches
        .value_of("username")
        .ok_or(ErrorKind::ConnectionString)?;
    let password = matches
        .value_of("password")
        .ok_or(ErrorKind::ConnectionString)?;
    let ctxt = ContextBuilder::default()
        .conn_string(conn_string.to_string())
        .username(username.to_string())
        .password(password.to_string())
        .build()?;

    match conn(&ctxt) {
        Ok(()) => {}
        Err(e) => {
            writeln!(io::stderr(), "{}", ctxt.db_context().get_error())?;
            return Err(e);
        }
    }

    Ok(0)
}
