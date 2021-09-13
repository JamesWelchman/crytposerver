use std::fs;
use std::thread::sleep;
use std::time::Duration;

use log::{error, info};

#[macro_use]
extern crate lazy_static;

mod config;
use config::{Mode, MODE, SECRET_DIR};

mod errors;
use errors::*;
mod server;

const HEXCHARS: &str = "0123456789abcdef";

fn sanitycheck_mode32() -> Result<()> {
    // We should 2^16 secret files, each containing
    // 2^16 secrets.
    let mut secret_file = SECRET_DIR.clone();
    for (((a, b), c), d) in HEXCHARS
        .chars()
        .zip(HEXCHARS.chars())
        .zip(HEXCHARS.chars())
        .zip(HEXCHARS.chars())
    {
        secret_file.push(format! {"{}{}{}{}", a, b, c, d});

        let metadata = fs::metadata(secret_file.to_str().unwrap())?;
        if metadata.len() != 32 * 2_u64.pow(16) {
            return Err(Box::new(Error::BadSecretFileMode32));
        }

        secret_file.pop();
    }

    Ok(())
}

fn sanitycheck_mode16() -> Result<()> {
    // We should have a single secret file
    // SECRET_DIR / '0000'
    let mut secret_file = SECRET_DIR.clone();
    secret_file.push("0000");

    let metadata = fs::metadata(secret_file.to_str().unwrap())?;
    if metadata.len() != 32 * 2_u64.pow(16) {
        return Err(Box::new(Error::BadSecretFileMode16));
    }

    Ok(())
}

fn sanitycheck_mode0() -> Result<()> {
    // We should have a single secret
    // SECRET_DIR / 'secret'
    let mut secret_file = SECRET_DIR.clone();
    secret_file.push("secret");

    let metadata = fs::metadata(secret_file.to_str().unwrap())?;
    if metadata.len() != 32 {
        return Err(Box::new(Error::BadSecretFileMode0));
    }

    Ok(())
}

fn run() -> Result<()> {
    // Perform startup checks
    info!("running on mode {:?}", *MODE);

    // Sanity check secret files
    info!(
        "sanity checking CRYPTOSERVER_SECRETDIR [{}]",
        SECRET_DIR.to_str().unwrap()
    );
    match *MODE {
        Mode::Mode0 => sanitycheck_mode0(),
        Mode::Mode16 => sanitycheck_mode16(),
        Mode::Mode32 => sanitycheck_mode32(),
    }?;

    loop {
        if let Err(e) = server::serve_forever() {
            error!("server is not running - {}", e.to_string());
            sleep(Duration::from_secs(1));
        }
    }
}

fn main() {
    env_logger::init();
    if let Err(e) = run() {
        error!("crytoserver crashed - {}", e.to_string());
    }
}
