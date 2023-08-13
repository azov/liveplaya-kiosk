use crate::core::Timestamp;
use crate::svc::*;

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

pub static DEFAULT_SERVER: &str = "rotate.aprs2.net:14580";

pub struct Client {
    server: String,
    reader: BufReader<TcpStream>,
}

impl Client {
    pub fn connect(server: impl ToString) -> Result<Self> {
        let server = server.to_string();
        let mut stream = TcpStream::connect(&server).map_err(|err| {
            Error::Other(format!(
                "failed to connect to APRS IS; server: {}, error:{}",
                server, err
            ))
        })?;

        log::info!("Connected to {}", server);

        //let hello = b"user N0CALL pass -1 vers brcmap 0.00 filter r/40.79608429483125/-119.19589964220306/200\r\n";
        let hello = b"user N0CALL pass -1 vers brcmap 0.00 filter r/37.371111/-122.0375/100 r/40.79608429483125/-119.19589964220306/100\r\n";

        stream.write(hello).map_err(|err| {
            Error::Other(format!(
                "failed to send data to APRS IS server;  server: {}, error:{}",
                server, err
            ))
        })?;

        Ok(Self {
            server,
            reader: std::io::BufReader::new(stream),
        })
    }

    pub fn read_next_packet(&mut self) -> Result<AprsPacket> {
        loop {
            let mut buf = String::new();
            match self.reader.read_line(&mut buf) {
                Ok(bytes_read) => {
                    if bytes_read > 0 {
                        log::debug!("Recv {:?}", buf);
                        let timestamp = Timestamp::now();
                        let data = buf.to_string();
                        return Ok(AprsPacket { received: timestamp, data });
                    }
                }
                Err(err) => {
                    return Err(Error::msg(format!(
                        "can't read next line from APRS IS server; server: {}, error: {}",
                        self.server, err
                    )));
                }
            }
        }
    }
}

pub fn spawn(tasks: Tasks, server: String) -> JoinHandle {
    std::thread::spawn(move || {
        let mut cli = Client::connect(server)?;
        loop {
            let p = cli.read_next_packet()?;
            tasks.submit(Task::PostAprs(p))?;
        }
    })
}
