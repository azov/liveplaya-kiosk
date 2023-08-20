#![allow(dead_code)]
use clap::Parser;
use std::path::PathBuf;

mod app;
mod bm;
mod brc;
mod brc2023;
pub mod aprs;
mod aprs_is;
mod aprs_tty;
// mod aprslog;
mod bmorg;
mod clockpos;
// mod jsonl;
mod err;
mod io;
mod motion;
mod svc;
mod util;
mod webapi;

use crate::err::Result;

const VERSION: &str = env!("CARGO_PKG_VERSION");

    /// Location of data files directory
    // #[arg(long, value_name = "DIR", env, alias = "dataroot")]
    // data: Option<std::path::PathBuf>, // this replaces --log and --digest


#[derive(clap::Parser)]
#[command(author, version, about, long_about = None, disable_colored_help = false)]
struct Args {
    /// Enable verbose log output
    #[arg(short, long, default_value_t = false, env, global = true)]
    verbose: bool,

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
        short = 'p',
        value_name = "PORT",
        default_value_t = 8080,
        env,
        alias = "httpport"
    )]
    httpport: u16,

    /// Serve web files from this directory
    #[arg(long, short, value_name = "URL", env, alias = "docroot")]
    wwwroot: Option<std::path::PathBuf>,

    /// APRS IS server URL & port (e.g. rotate.aprs2.net:14580)
    #[arg(long, value_name = "URL", env)]
    aprsis: Option<String>,

    /// Event log file
    #[arg(long, short = 'l', value_name = "FILENAME", env)]
    eventlog: Option<PathBuf>,

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
            args.tty,
            args.baudrate,
            args.aprsis,
            args.eventlog,
        )
        .await
    }
}
