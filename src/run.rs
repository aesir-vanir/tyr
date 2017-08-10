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
use mimir::enums::ODPIFetchMode::Next;
use mimir::{flags, Connection};
use std::ffi::CString;

/// Connect to the database.
fn conn(ctxt: &Context) -> Result<()> {
    use std::io::{self, Write};
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

    let user_tables = conn.prepare_stmt(Some("select table_name from user_tables"), None, false)?;

    let _cols = user_tables.execute(flags::DPI_MODE_EXEC_DEFAULT)?;
    let user_tables_query_info = user_tables.get_query_info(1)?;
    writeln!(
        io::stdout(),
        "Cols: {}",
        user_tables_query_info.oracle_type_num()
    )?;
    let (_buffer_row_index, _num_rows_fetched, _more_rows) = user_tables.fetch_rows(10)?;
    user_tables.scroll(Next, 0, 0)?;
    user_tables.close(None)?;
    Ok(())
}

/// CLI Runtime
pub fn run() -> Result<i32> {
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
    conn(&ctxt)?;

    Ok(0)
}
