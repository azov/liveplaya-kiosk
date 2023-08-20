use crate::err::{Error, Result};

pub fn print_ttys() -> Result<()> {
    for p in tokio_serial::available_ports().map_err(|e| Error::msg(e))? {
        println!("{:?}", p);
    }
    Ok(())
}
