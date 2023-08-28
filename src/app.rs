use crate::{
    aprs_is, aprs_tty,
    err::{Error, LogResult, Result},
    io,
    svc::jsonlog::JsonLog,
    util::time::{Timespan, Timestamp},
    webapi,
};
use std::collections::HashMap;
use tokio::sync::mpsc;

mod get_view;
mod post_aprs;
mod print_ttys;

pub use print_ttys::*;

pub async fn run(
    http_port: u16,
    www_root: Option<std::path::PathBuf>,
    tty: Option<String>,
    baudrate: Option<u16>,
    aprsis_server: Option<String>,
    eventlog: Option<std::path::PathBuf>,
) -> Result<()> {
    let mut tasks = tokio::task::JoinSet::new();

    // Create store
    let eventlog = eventlog.unwrap_or("/tmp/lpkiosk-events.log".into());
    let store = JsonLog::<io::store::Record>::new(eventlog);

    // Create I/O channels
    let (aprs_dta_tx, aprs_dta_rx) = mpsc::channel::<String>(1024);
    let (user_evt_tx, user_evt_rx) = mpsc::channel::<io::user::Event>(1024);

    // Spawn service tasks
    tasks.spawn(Server::new(user_evt_rx, aprs_dta_rx, store).run());

    // APRS TTY
    if let Some(tty) = tty {
        tasks.spawn(aprs_tty::read(
            tty,
            baudrate.unwrap_or(9600) as u32,
            aprs_dta_tx.clone(),
        ));
    }

    // APRS IS
    tasks.spawn(aprs_is::read(
        aprsis_server.unwrap_or(aprs_is::DEFAULT_SERVER.into()),
        aprs_dta_tx.clone(),
    ));

    // Actix handles its own shutdown and we'll piggy back on that (also, it
    // doesn't seem to work as a spawned task, so we kinda have to)
    if let Err(e) = webapi::run(http_port, www_root, user_evt_tx).await {
        log::error!("{}", e);
    }

    // Wait for clean shutdown
    tasks.abort_all();
    while let Some(_) = tasks.join_next().await {}
    log::info!("app exited");
    Ok(())
}

pub struct Server {
    brc: crate::brc::BlackRockCity,
    aprs_cache: crate::aprs::Log,
    pois_by_call: HashMap<&'static str, &'static io::site::Poi>,

    user_evt_rx: mpsc::Receiver<io::user::Event>,
    aprs_dta_rx: mpsc::Receiver<String>,

    store: JsonLog<io::store::Record>,
}

impl Server {
    pub fn new(
        user_evt_rx: mpsc::Receiver<io::user::Event>,
        aprs_dta_rx: mpsc::Receiver<String>,
        store: JsonLog<io::store::Record>,
    ) -> Self {
        let brc = crate::brc2023::get();
        let aprs_cache = crate::aprs::Log::new();
        let pois_by_call = crate::brc2023::POIS
            .iter()
            .map(|poi| (poi.call, poi))
            .collect();
        Self {
            brc,
            aprs_cache,
            pois_by_call,
            user_evt_rx,
            aprs_dta_rx,
            store,
        }
    }

    pub async fn run(mut self) -> Result<()> {
        self.preload().await.log_result();

        log::debug!("entering server mainloop");
        loop {
            tokio::select! {
                Some(evt) = self.user_evt_rx.recv() => self.process_user_event(evt).await.log_result(),
                Some(data) = self.aprs_dta_rx.recv() => self.process_aprs_data(data).await.log_result(),
            }
        }
    }

    pub async fn process_user_event(&mut self, evt: io::user::Event) -> Result<()> {
        match evt {
            io::user::Event::ViewRequest(_query, res) => {
                let view_res = self.view().await;
                res.send(view_res).map_err(|_| Error::Disconnected)
            }
        }
    }

    pub async fn process_aprs_data(&mut self, data: String) -> Result<()> {
        let now = Timestamp::now();
        self.store
            .write(now, &io::store::Record::AprsPacket { data: data.clone() })
            .await
            .log_result();
        self.post_aprs(now, data).await
    }

    pub async fn preload(&mut self) -> Result<()> {
        log::info!("preloading data...");
        let mut cnt = 0;
        let span = Timespan::week_until_now();
        match self.store.query(span).await {
            Ok(records) => {
                for (ts, rec) in records {
                    match rec {
                        io::store::Record::AprsPacket { data } => {
                            cnt += 1;
                            self.post_aprs(ts, data).await.log_result();
                        }
                    };
                }
            }
            Err(e) => Err(e).log_result(),
        }
        log::info!("preloaded {} items", cnt);
        Ok(())
    }
}
