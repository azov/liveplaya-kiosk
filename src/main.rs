#![type_length_limit = "4194304"]
use clap::{crate_version, value_t, App, AppSettings, Arg};
use liveplaya::logger;
use log::info;

mod webui;

fn main() {
    const APPNAME: &'static str = "LivePlaya Kiosk";

    let args = App::new(APPNAME)
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::UnifiedHelpMessage)
        .setting(AppSettings::DeriveDisplayOrder)
        .version(crate_version!())
        .version_message("Print version and exit")
        .help_message("Print help message and exit")
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .global(true)
                .help("Enable verbose log output"),
        )
        .arg(
            Arg::with_name("nocolor")
                .long("nocolor")
                .global(true)
                .help("Disable colors in log output"),
        )
        .arg(
            Arg::with_name("timestamp")
                .long("timestamp")
                .global(true)
                .help("Enable timestamps in log output"),
        )
        .arg(
            Arg::with_name("log")
                .long("log")
                .takes_value(true)
                .help("APRS log file with all received packets"),
        )
        // .arg(Arg::with_name("digest")
        //      .long("digest")
        //      .takes_value(true)
        //      .help("APRS log file with a filtered set of packets relevant to the current state"))
        .arg(
            Arg::with_name("aprsis")
                .long("aprsis")
                .takes_value(true)
                .help("APRS IS server URL & port"),
        )
        .arg(
            Arg::with_name("tty")
                .long("tty")
                .takes_value(true)
                .help("Read from given serial device"),
        )
        .arg(
            Arg::with_name("baudrate")
                .long("baudrate")
                .takes_value(true)
                .help("Baud rate"),
        )
        .arg(
            Arg::with_name("httpport")
                .long("httpport")
                .takes_value(true)
                .help("Listen the a given port"),
        )
        .arg(
            Arg::with_name("docroot")
                .long("docroot")
                .takes_value(true)
                .help("Serve files from given directory"),
        )
        .get_matches();

    logger::init(
        args.occurrences_of("verbose") > 0,
        args.occurrences_of("nocolor") > 0,
        args.occurrences_of("timestamp") > 0,
    );

    let http_port = value_t!(args.value_of("httpport"), u16).unwrap_or(8080);
    let log_file = args.value_of("log");
    let tty = args.value_of("tty");
    let baudrate = value_t!(args.value_of("baudrate"), u32).unwrap_or(9600);
    let aprsis_server = args.value_of("aprsis");

    info!("{} v{}", APPNAME, crate_version!());

    let mut server = liveplaya::server::new();

    server.http_port(http_port).configure_ui(webui::configure);

    if let Some(tty) = tty {
        server.aprs_tnc(tty, baudrate);
    }

    if let Some(path) = log_file {
        server.log(path);
    }

    if let Some(addr) = aprsis_server {
        server.aprs_is(addr);
    }

    server.run();
}
