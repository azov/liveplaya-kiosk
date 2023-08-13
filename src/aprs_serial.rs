use crate::core::Timestamp;
use crate::err::*;
use crate::svc::*;

use serial::{self, SerialPort};
use std::io::{BufRead, BufReader, Error as IoError, ErrorKind as IoErrorKind};

fn is_timeout(res: std::result::Result<String, IoError>) -> bool {
    if let Err(e) = res {
        if e.kind() == IoErrorKind::TimedOut {
            return true
        }
    }
    false
}

pub fn spawn(tasks: Tasks, tty: String, baudrate: u16) -> JoinHandle {
    std::thread::spawn(move || run(tasks, tty, baudrate))
}

pub fn run(tasks: Tasks, tty: String, baudrate: u16) -> Result<()> {
    let settings = serial::PortSettings {
        baud_rate: serial::BaudRate::from_speed(baudrate as usize),
        char_size: serial::Bits8,
        parity: serial::ParityNone,
        stop_bits: serial::Stop1,
        flow_control: serial::FlowNone,
    };

    'outer: loop {
        log::debug!(
            "Opening {}, baud rate {}...",
            tty,
            settings.baud_rate.speed()
        );
    
        let mut serport = serial::open(&tty).map_err(|e| Error::OpenTTY {
            tty: tty.clone(),
            err: e.to_string(),
        })?;
        serport.configure(&settings).map_err(|e| Error::OpenTTY {
            tty: tty.clone(),
            err: e.to_string(),
        })?;
        // serport
        //     .set_timeout(Duration::from_secs(1))
        //     .map_err(|err| Error::Open {
        //         tty: tty.clone(),
        //         err,
        //     })?;
    
        log::info!("Opened {}, baud rate {}", tty, settings.baud_rate.speed());
    
        'inner: for line in BufReader::new(serport).lines() {
            match line {
                Ok(line) => {
                    let received = Timestamp::now();
                    let data = line;
                    tasks.submit(Task::PostAprs(AprsPacket { received, data}))?;
                },
                Err(e) => {
                    if e.kind() == IoErrorKind::TimedOut {
                        continue 'inner; // Ignore timeouts
                    }
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    continue 'outer;
                }
            }
        }
    }
}
