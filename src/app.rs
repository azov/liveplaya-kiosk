pub use crate::core::Data;
pub use crate::svc::*;

use crate::aprs_is;
use crate::aprs_serial;
use crate::webapi;

pub fn run(
    http_port: u16,
    aprsis: Option<String>,
    tty: Option<String>,
    baudrate: Option<u16>,
) -> Result<()> {
    let worker = Worker::new();
    let (cli, worker_hdl) = worker.spawn(8);
    let mut handles = Vec::new();
    handles.push(worker_hdl);
    
    let aprsis = aprsis.unwrap_or(aprs_is::DEFAULT_SERVER.into());
    if aprsis != "no" {
        handles.push(aprs_is::spawn(cli.clone(), aprsis));       
    }
    if let Some(tty) = tty {
        handles.push(aprs_serial::spawn(cli.clone(), tty, baudrate.unwrap_or(9600)));       
    }
    set_shutdown_handler(cli.clone());
    webapi::Server::new(http_port, cli).run()?;

    for hdl in handles {
        let _ = hdl.join();
    }
    log::info!("app exited");
    Ok(())
}

pub fn set_shutdown_handler(cli: Tasks) {
    ctrlc::set_handler(move || {
        let _ = cli.submit(Task::Shutdown);
    })
    .expect("error setting Ctrl-C handler");
}

struct Worker {
    data: Data,
}
impl Worker {
    pub fn new() -> Self {
        let data = crate::core::Data::new();
        Self { data }
    }

    pub fn post_aprs(&mut self, packet: AprsPacket) -> Result<()> {
        self.data.update_aprs(packet.received, packet.data)
    }

    pub fn render(&self, q: Query) -> Result<View> {
        let v = self.data.snapshot(q.feature);
        Ok(View {
            name: v.name(),
            unix_timestamp_ms: v.timestamp().as_unix_millis(),
            description: v.description(),
            map: v.to_maplibre_style(),
        })
    }
}

impl TaskProcessor<Task> for Worker {
    fn process(&mut self, task: Task) -> Result<()> {
        match task {
            Task::Shutdown => Err(Error::msg("shutting down")),
            Task::GetView(req) => req.fulfill(|q| self.render(q)),
            Task::PostAprs(packet) => self.post_aprs(packet),
        }
    }
}
