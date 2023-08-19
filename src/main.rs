#![allow(dead_code)]
use clap::Parser;
use std::path::PathBuf;

mod app;
mod bm;
mod brc;
// mod core;
mod bmorg;
pub mod aprs;
mod aprs_is;
mod aprs_log;
mod aprs_serial;
mod clockpos;
mod io;
mod err;
mod motion;
mod util;
mod webapi;
mod worker;

use crate::err::Result;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None, disable_colored_help = false)]
struct Args {
    /// Enable verbose log output
    #[arg(short, long, default_value_t = false, env, global = true)]
    verbose: bool,

    /// Location of data files directory
    #[arg(long, value_name = "DIR", env, alias = "dataroot")]
    data: Option<std::path::PathBuf>, // this replaces --log and --digest

    /// APRS log file
    #[arg(long, value_name = "FILENAME", env)]
    aprslog: Option<PathBuf>,

    /// APRS IS server URL & port
    #[arg(long, value_name = "URL", env)]
    aprsis: Option<String>,

    /// Read from this serial device
    #[arg(long, value_name = "URL", env)]
    tty: Option<String>,

    /// TTY baud rate
    #[arg(long, value_name = "URL", env)]
    baudrate: Option<u16>,

    /// Print available serial ports and exit
    #[arg(long, default_value_t = false)]
    print_ttys: bool,

    /// Serve HTTP requests from this port
    #[arg(
        long,
        value_name = "PORT",
        default_value_t = 8080,
        env,
        alias = "httpport"
    )]
    httpport: u16,

    /// Serve web files from this directory
    #[arg(long, value_name = "URL", env, alias = "docroot")]
    wwwroot: Option<std::path::PathBuf>,
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose {
        std::env::set_var("RUST_LOG", "debug");
    } else if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    log::debug!("debug logging enabled");
    log::info!("version: {}", VERSION);

    if args.print_ttys {
        app::print_ttys()
    } else {
        app::run(
            args.httpport,
            args.wwwroot,
            args.aprsis,
            args.aprslog,
            args.tty,
            args.baudrate,
        )
        .await
    }
}
