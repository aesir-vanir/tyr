// Copyright (c) 2017 tyr developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `cargo-tyr` runtime.
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};
use error::{ErrorKind, Result};
use std::fmt;
use std::process::{Command, Stdio};
use term;
use tmpl::Templates;

/// output level
#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Level {
    /// Output everything
    Trace = 0,
    /// Everything but TRACE
    Debug = 1,
    /// Everything but TRACE and DEBUG
    Info = 2,
    /// Warn and above only.
    Warn = 3,
    // /// Error and above only.
    // Error = 4,
    // /// Only fatal messages are output.
    // Fatal = 5,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Level::Trace => write!(f, "Trace"),
            Level::Debug => write!(f, "Debug"),
            Level::Info => write!(f, "Info"),
            Level::Warn => write!(f, "Warn"),
        }
    }
}

/// Parse the args, and execute the generated commands.
pub fn run() -> Result<i32> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Create a Rust ORM library for Oracle databases")
        .setting(AppSettings::GlobalVersion)
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("tyr")
                .arg(
                    Arg::with_name("vcs")
                        .long("vcs")
                        .value_name("VCS")
                        .help(
                            "Initialize a new repository for the given version control system
                        or do not initialize any version control at all, overriding a
                        global configuration.",
                        )
                        .possible_values(&["git", "hg", "pijul", "fossil", "none"])
                        .default_value("git")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("name")
                        .long("name")
                        .value_name("NAME")
                        .help("Set the resulting package name, defaults to the value of <path>.")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("color")
                        .long("color")
                        .value_name("WHEN")
                        .help("Coloring")
                        .possible_values(&["auto", "always", "never"])
                        .default_value("auto")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("frozen")
                        .long("frozen")
                        .conflicts_with("locked")
                        .help("Require Cargo.lock and cache are up to date"),
                )
                .arg(Arg::with_name("locked").long("locked").help("Require Cargo.lock is up to date"))
                .arg(
                    Arg::with_name("verbose")
                        .short("v")
                        .multiple(true)
                        .help("Use verbose output (-vv very verbose/build.rs output)"),
                )
                .arg(
                    Arg::with_name("quiet")
                        .short("q")
                        .long("quiet")
                        .conflicts_with("verbose")
                        .help("No output printed to stdout"),
                )
                .arg(
                    Arg::with_name("license")
                        .long("license")
                        .value_name("TYPE")
                        .help("Specify licensing to include in the generated output.")
                        .possible_values(&["both", "mit", "apache", "none"])
                        .default_value("both")
                        .takes_value(true),
                )
                .arg(Arg::with_name("no-readme").long("no-readme").help("Turn off README.md generation."))
                .arg(
                    Arg::with_name("no-latest")
                        .long("no-latest")
                        .help("Turn off the crates.io query for the latest version (use defaults)."),
                )
                .arg(Arg::with_name("path").takes_value(true).required(true)),
        )
        .get_matches();

    let tyr_matches = matches.subcommand_matches("tyr").ok_or_else(|| ErrorKind::SubCommand)?;
    let (_path, name, level, cargo_new_args) = setup_cargo_new_args(&tyr_matches)?;

    let readme = !tyr_matches.is_present("no-readme");
    let query = !tyr_matches.is_present("no-latest");

    let (mit, apache) = match tyr_matches.value_of("license").ok_or_else(|| ErrorKind::License)? {
        "both" => (true, true),
        "mit" => (true, false),
        "apache" => (false, true),
        "none" => (false, false),
        _ => return Err(ErrorKind::License.into()),
    };

    let _template = Templates::new(name, mit, apache, readme, query);

    let mut cargo_new = Command::new("cargo").stdout(Stdio::null()).stderr(Stdio::null()).args(&cargo_new_args).spawn()?;
    let ecode = cargo_new.wait()?;

    if !ecode.success() {
        return Ok(ecode.code().ok_or_else(|| ErrorKind::ExitCode)?);
    }

    let msg = format!("Created ORM library `{}` project", name);
    info("Created", &msg, &level)?;

    Ok(0)
}

/// Setup the `cargo new` comand arguments.
fn setup_cargo_new_args<'a>(matches: &'a ArgMatches) -> Result<(&'a str, &'a str, Level, Vec<&'a str>)> {
    let mut argv = Vec::new();
    argv.push("new");

    if matches.is_present("frozen") {
        argv.push("--frozen");
    }

    if matches.is_present("locked") {
        argv.push("--locked");
    }

    let level = if matches.is_present("quiet") {
        argv.push("--quiet");
        Level::Warn
    } else {
        match matches.occurrences_of("verbose") {
            0 => Level::Info,
            1 => {
                argv.push("-v");
                Level::Debug
            }
            2 | _ => {
                argv.push("-vv");
                Level::Trace
            }
        }
    };

    if let Some(color) = matches.value_of("color") {
        argv.push("--color");
        argv.push(color);
    }

    if let Some(vcs) = matches.value_of("vcs") {
        argv.push("--vcs");
        argv.push(vcs);
    }

    let path = if let Some(path) = matches.value_of("path") {
        path
    } else {
        return Err(ErrorKind::Path.into());
    };

    let name = if let Some(name) = matches.value_of("name") {
        argv.push("--name");
        argv.push(name);
        argv.push(path);
        name
    } else {
        argv.push(path);
        path
    };
    Ok((path, name, level, argv))
}

/// Log a `cargo` formatted message to the terminal.
fn log_message(verb: &str, message: &str) -> Result<()> {
    let mut t = term::stdout().ok_or(ErrorKind::TermCommand)?;
    t.fg(term::color::BRIGHT_GREEN)?;
    t.attr(term::Attr::Bold)?;
    write!(t, "{:>12}", verb)?;
    t.reset()?;
    writeln!(t, " {}", message)?;
    t.flush()?;
    Ok(())
}

/// Log a debug level message to the terminal.
fn debug(verb: &str, message: &str, level: &Level) -> Result<()> {
    if *level <= Level::Debug {
        log_message(verb, message)?;
    }
    Ok(())
}

/// Log an info level message to the terminal.
fn info(verb: &str, message: &str, level: &Level) -> Result<()> {
    if *level <= Level::Info {
        log_message(verb, message)?;
    }
    Ok(())
}
