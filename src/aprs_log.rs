use chrono::{DateTime, Utc, format::ParseError as TimeParseError};
use data::formats::aprs::RawPacket;
use std::fs::{File};
use std::fs::{OpenOptions, create_dir_all};
use std::io::{BufReader, BufRead, Error as IoError};
use std::io::{BufWriter, Write};
use std::path::Path;
use logger::OkOrLog;


#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "log line {} too short: [{}]", linenum, line)]
    LineTooShort{ linenum: usize, line: String },

    #[fail(display = "log line {} has invalid time [{}]: {}", linenum, raw, err)]
    ParseTime{ linenum: usize, raw: String, err: TimeParseError },

    #[fail(display = "log line {} contains invalid UTF-8", linenum)]
    Utf8{ linenum: usize },

    #[fail(display = "failed to read log: {}", err)]
    Read{ err: IoError },

    #[fail(display = "failed to write log: {}", err)]
    Write{ err: IoError },

    #[fail(display = "failed to create log directory: {}", err)]
    MkDir{ err: IoError },
}

fn parse_utc(linenum: usize, raw: &str) -> Result<DateTime<Utc>, Error> {
	// There must be an easier way to do it!
	Ok(DateTime::<Utc>::from_utc(DateTime::parse_from_rfc3339(raw)
		.map_err(|err| Error::ParseTime{linenum, raw: raw.to_string(), err})?.naive_utc(), Utc))
}

fn parse(path: &str, linenum: usize, line : String) -> Option<RawPacket> {
	if line.len() < 22 {
		debug!("{}", Error::LineTooShort{ linenum, line });
		return None
	}
	
	let raw_time = &line[0..20];
	let raw_data = &line[21..];

	match parse_utc(linenum, raw_time) {
		Err(err) => {
			debug!("{}", err);
			None
		},
		Ok(time) => Some(RawPacket{
			source: format!("{} line {}", path, linenum),
			received: time, 
			data: raw_data.as_bytes().to_vec()}),
	}
}

pub fn read_mem(data : &'static str) -> impl Iterator<Item = RawPacket> {
	data.lines()
		.enumerate()
        .filter_map(move |(n,l)| parse(":memory:", n, l.to_string()))
}

pub fn read(path : &str) -> Result<impl Iterator<Item = RawPacket>, Error> {
	info!("Reading {}...", path);
	let file = File::open(path)
		.map_err(|err| Error::Read{err})?;
	let path = path.to_string();
	let res = BufReader::new(file)
		.lines()
		.enumerate()
        .filter_map(|(n,l)| l.ok_or_log().map(|l| (n,l)))
        .filter_map(move |(n,l)| parse(&path, n, l))
    ;
	Ok(res)
}

pub fn write(path: &str, packet: &RawPacket) -> Result<(), Error> {
	if let Some(dir) = Path::new(path).parent() {
		create_dir_all(dir).map_err(|err| Error::MkDir{err})?;
	}
	let mut writer = BufWriter::new(OpenOptions::new().create(true).append(true).open(path)
		.map_err(|err| Error::Write{ err})?);

	writer.write(&packet.received.format("%Y-%m-%dT%H:%M:%SZ ").to_string().as_bytes())
		.and(writer.write(&packet.data))
		.and(writer.write("\n".as_bytes()))
		.and(writer.flush())
		.map_err(|err| Error::Write{ err })?;
	Ok(())
}

