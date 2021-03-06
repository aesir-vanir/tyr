// Copyright (c) 2017 tyr developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `tyr` errors
error_chain! {
    foreign_links {
        Io(::std::io::Error);
        FromUtf8(::std::string::FromUtf8Error);
        Mimir(::mimir::Error);
        Mustache(::mustache::Error);
        Term(::term::Error);
    }
    errors {
        ConnectionString {
            description("The connection string is a required command line argument!")
            display("The connection string is a required command line argument!")
        }
        Max {
            description("")
            display("")
        }
        Stdout {
            description("Unable to open the stdout terminal for writing!")
            display("Unable to open the stdout terminal for writing!")
        }
    }
}
