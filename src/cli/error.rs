// Copyright (c) 2017 tyr developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `cargo-tyr` errors

error_chain!{
    foreign_links {
        FromUtf8(::std::string::FromUtf8Error);
        Io(::std::io::Error);
        Mustache(::mustache::Error);
        Term(::term::Error);
    }

    errors {
        ExitCode {
           description("")
            display("")
        }
        License {
            description("")
            display("")
        }
        Path {
            description("")
            display("")
        }
        SubCommand {
            description("")
            display("")
        }
        TermCommand {
            description("")
            display("")
        }
    }
}
