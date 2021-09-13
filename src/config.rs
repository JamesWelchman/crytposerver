use std::env;
use std::path::PathBuf;

pub const DEFAULT_BIND: &str = "0.0.0.0:8080";

lazy_static! {
    // Mode is the mode in which we run
    // We either run in 0, 16 or 32 mode
    // depending on the number of secrets used.
    pub static ref MODE: Mode = get_mode();
}

lazy_static! {
    // secret_dir corresponds to the environment variable
    // CRYPTOSERVER_SECRETDIR. It is where the server looks
    // to find the secrets.
    pub static ref SECRET_DIR: PathBuf = get_secret_dir();
}

#[derive(Copy, Clone, Debug)]
pub enum Mode {
    Mode0,
    Mode16,
    Mode32,
}

pub fn get_mode() -> Mode {
    let mut mode = Mode::Mode0;

    for (k, v) in env::vars() {
        if k == "CRYPTOSERVER_MODE" {
            mode = match v.as_ref() {
                "MODE0" => Mode::Mode0,
                "MODE16" => Mode::Mode16,
                "MODE32" => Mode::Mode32,
                _ => Mode::Mode0,
            };
        }
    }

    mode
}

pub fn get_secret_dir() -> PathBuf {
    let mut path = PathBuf::from("/secrets".to_string());
    for (k, v) in env::vars() {
        if k == "CRYPTOSERVER_SECRETDIR" {
            path = PathBuf::from(v);
        }
    }

    // Sanity checking the secret file is delayed
    // as we can't guarantee that get_secret_dir
    // is called before get_mode and therefore it
    // might not be set.

    path
}
