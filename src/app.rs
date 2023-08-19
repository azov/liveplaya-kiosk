use crate::{
    aprs_is, aprs_log, aprs_serial,
    err::{Error, Result},
    io,
    util::{time::*, twoway},
    webapi, worker,
};
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub async fn run(
    http_port: u16,
    www_root: Option<std::path::PathBuf>,
    aprsis_server: Option<String>,
    aprs_log: Option<PathBuf>,
    tty: Option<String>,
    baudrate: Option<u16>,
) -> Result<()> {
    let stop = CancellationToken::new();
    let (aprs_tx, aprs_rx) = mpsc::channel::<String>(1024);
    let (log_tx, log_rx) = mpsc::channel::<(Timestamp, String)>(1024);
    let (query_tx, query_rx) = twoway::channel::<io::Query, io::View>(1024);
    let mut handles = Vec::new();

    // set shutdown handler
    {
        let stop = stop.clone();
        ctrlc::set_handler(move || {
            log::info!("interrupted");
            stop.cancel();
        })
        .expect("error setting interrupt handler");
    }

    // spawn aprs_is reader
    if aprsis_server.as_ref().map(|s| s != "none").unwrap_or(true) {
        handles.push(tokio::spawn(aprs_is::run(
            aprsis_server.unwrap_or(aprs_is::DEFAULT_SERVER.into()),
            aprs_tx.clone(),
            stop.clone(),
        )));
    } else {
        log::debug!("not connecting to aprs_is because the server is 'none'");
    }

    // spawn tty reader
    if let Some(tty) = tty {
        handles.push(tokio::spawn(aprs_serial::run(
            tty,
            baudrate.unwrap_or(9600) as u32,
            aprs_tx.clone(),
            stop.clone(),
        )));
    } else {
        log::debug!("not reading tty");
    }

    // spawn log writer
    if let Some(path) = aprs_log {
        handles.push(tokio::spawn(aprs_log::run_writer(
            path,
            log_rx,
            stop.clone(),
        )));
    } else {
        log::debug!("not writing log");
    }

    // spawn worker
    handles.push(tokio::spawn(worker::run(
        query_rx,
        aprs_rx,
        log_tx,
        stop.clone(),
    )));

    // run web server (this doesn't work with tokio::spawn)
    if let Err(e) = webapi::run(http_port, www_root, query_tx, stop.clone()).await {
        log::error!("{}", e);
    };

    for h in handles {
        let _ = h.await;
    }
    log::info!("app exited");
    Ok(())
}

pub fn print_ttys() -> Result<()> {
    for p in tokio_serial::available_ports().map_err(|e| Error::msg(e))? {
        println!("{:?}", p);
    }
    Ok(())
}
