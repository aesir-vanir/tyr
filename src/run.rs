// Copyright (c) 2017 tyr developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `tyr` runtime
use chrono::{DateTime, Utc};
use clap::{App, Arg};
use context::{Context, ContextBuilder};
use error::{ErrorKind, Result};
use mimir::enums::ODPINativeTypeNum::Bytes;
use mimir::enums::ODPIOracleTypeNum::Varchar;
use mimir::{flags, Connection, Data};
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

#[derive(Clone, Debug, Default, Getters, PartialEq, Setters)]
/// Column information relevant for `tyr`.
pub struct ColumnInfo {
    /// The column name.
    #[set = "pub"]
    // #[get = "pub"]
    name: String,
    /// The column data type.
    #[set = "pub"]
    // #[get = "pub"]
    data_type: Option<String>,
    /// The column data type modifier.
    #[set = "pub"]
    // #[get = "pub"]
    data_type_mod: Option<String>,
    /// The column data length.
    #[set = "pub"]
    // #[get = "pub"]
    data_length: f64,
    /// Is the column nullable?
    #[set = "pub"]
    // #[get = "pub"]
    nullable: Option<bool>,
    /// Last analysis time.
    #[set = "pub"]
    // #[get = "pub"]
    last_analyzed: Option<DateTime<Utc>>,
}

/// Generate a `String` from an optional `Display` or "(null)" if None.
fn string_or_null<T: fmt::Display>(opt_val: &Option<T>) -> String {
    if let Some(ref val) = *opt_val {
        val.to_string()
    } else {
        "(null)".to_string()
    }
}

impl fmt::Display for ColumnInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}|{}|{}|{}|{}|{}",
            self.name,
            string_or_null(&self.data_type),
            string_or_null(&self.data_type_mod),
            self.data_length,
            string_or_null(&self.nullable),
            string_or_null(&self.last_analyzed)
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

    let _cols = user_tables.execute(flags::DPI_MODE_EXEC_DEFAULT)?;
    let (mut found, _) = user_tables.fetch()?;
    let mut table_names: HashMap<String, Vec<ColumnInfo>> = HashMap::new();

    while found {
        let (_id_type, id_ptr) = user_tables.get_query_value(1)?;
        let data: Data = id_ptr.into();
        table_names.insert(data.get_string(), Default::default());
        let (f, _) = user_tables.fetch()?;
        found = f;
    }

    for (table, col_info_vec) in &mut table_names {
        let table_desc = conn.prepare_stmt(Some(DESC), None, false)?;
        let table_name_var = conn.new_var(Varchar, Bytes, 1, 256, false, false)?;
        table_name_var.set_from_bytes(0, table)?;
        table_desc.bind_by_name(":table_name", &table_name_var)?;

        let _cols = table_desc.execute(flags::DPI_MODE_EXEC_DEFAULT)?;
        let (mut found, _buffer_row_index) = table_desc.fetch()?;

        while found {
            let (_, name_ptr) = table_desc.get_query_value(2)?;
            let name_data: Data = name_ptr.into();
            let (_, data_type_ptr) = table_desc.get_query_value(3)?;
            let data_type_data: Data = data_type_ptr.into();
            let (_, data_type_mod_ptr) = table_desc.get_query_value(4)?;
            let data_type_mod_data: Data = data_type_mod_ptr.into();
            let (_, data_type_owner_ptr) = table_desc.get_query_value(5)?;
            let _data_type_owner_data: Data = data_type_owner_ptr.into();
            let (_, data_length_ptr) = table_desc.get_query_value(6)?;
            let data_length_data: Data = data_length_ptr.into();
            let (_, nullable_ptr) = table_desc.get_query_value(9)?;
            let nullable_data: Data = nullable_ptr.into();
            let (_, last_analyzed_ptr) = table_desc.get_query_value(18)?;
            let last_analyzed_data: Data = last_analyzed_ptr.into();

            let mut col_info: ColumnInfo = Default::default();

            if !name_data.null() {
                let col_name = name_data.get_string();
                col_info.set_name(col_name);
            }
            not_null!(nullable_data, {
                let nullable = nullable_data.get_string() == "Y";
                col_info.set_nullable(Some(nullable));
            });
            not_null!(data_type_data, {
                let data_type = data_type_data.get_string();
                col_info.set_data_type(Some(data_type));
            });
            not_null!(data_type_mod_data, {
                let data_type_mod = data_type_mod_data.get_string();
                col_info.set_data_type_mod(Some(data_type_mod));
            });
            if !data_length_data.null() {
                col_info.set_data_length(data_length_data.get_double());
            }
            not_null!(last_analyzed_data, {
                let last_analyzed = last_analyzed_data.get_utc();
                col_info.set_last_analyzed(Some(last_analyzed));
            });
            col_info_vec.push(col_info);

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
