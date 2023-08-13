#![allow(dead_code)]
use clap::Parser;

mod core;

mod app;
pub mod aprs;
mod aprs_is;
mod aprs_serial;
mod clockpos;
mod data;
mod err;
mod geo;
pub mod io;
mod motion;
pub(crate) mod sync;
mod svc;
mod time;
mod units;
mod util;
mod webapi;

use crate::core::Result;


const VERSION: &str = env!("CARGO_PKG_VERSION");


#[derive(clap::Parser)]
#[command(author, version, about, long_about = None, disable_colored_help = false)]
struct Args {
    /// Enable verbose log output
    #[arg(short, long, default_value_t = false, env, global = true)]
    verbose: bool,

    /// Enable timestamps log output
    #[arg(long, default_value_t = false, global = true)]
    log_timestamps: bool,

    /// Location of data files directory
    #[arg(long, value_name = "DIR", env, alias = "dataroot")]
    data: Option<std::path::PathBuf>, // this replaces --log and --digest

    /// APRS IS server URL & port
    #[arg(long, value_name = "URL", env)]
    aprsis: Option<String>,

    /// Read from this serial device
    #[arg(long, value_name = "URL", env)]
    tty: Option<String>,

    /// TTY baud rate
    #[arg(long, value_name = "URL", env)]
    baudrate: Option<u16>,

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

pub fn main() -> Result<()> {
    let args = Args::parse();

    if args.verbose {
        std::env::set_var("RUST_LOG", "debug");
    } else if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    log::debug!("debug logging enabled");
    log::info!("version: {}", VERSION);

    app::run(args.httpport, args.aprsis, args.tty, args.baudrate)
}

