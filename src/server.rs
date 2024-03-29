use std::env;

use crate::config::{Mode, DEFAULT_BIND, MODE, SECRET_DIR};
use crate::errors::*;

use async_std::fs::OpenOptions;
use async_std::io::SeekFrom;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use hmac::{Hmac, Mac, NewMac};
use http_types::{Method, Response, StatusCode};
use log::{debug, error, info};
use murmurhash32::murmurhash3;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn serve_forever() -> Result<()> {
    use std::net::TcpListener;

    let addr = get_addr();

    // Attempt to bind to the port
    let listener = TcpListener::bind(&addr)?;
    info!("server listening on [{}]", addr);
    task::block_on(accept_loop(listener))
}

async fn accept_loop(listener: std::net::TcpListener) -> Result<()> {
    // Convert to async listener
    let listener = TcpListener::from(listener);

    while let Some(stream) = listener.incoming().next().await {
        let stream = stream?;
        task::spawn(connection_loop(stream));
    }

    Ok(())
}

async fn connection_loop(stream: TcpStream) {
    if let Err(e) = connection_loop_run(stream).await {
        error!("problem with connection - {}", e.to_string())
    }
}

async fn connection_loop_run(stream: TcpStream) -> Result<()> {
    // Wait for the Method and the Path to arrive
    async_h1::accept(stream.clone(), |mut req| async move {
        // We only accept POST request
        if req.method() != Method::Post {
            error!("only POST method is valid");
            return Ok(Response::new(StatusCode::BadRequest));
        }
        if req.url().path() != "/hmac" {
            error!("only /hmac endpoint is valid");
            return Ok(Response::new(StatusCode::BadRequest));
        }

        // Okay, take the body
        let data = req.body_bytes().await?;
        if data.is_empty() {
            error!("empty body is not allowed");
            return Ok(Response::new(StatusCode::BadRequest));
        }

        // Compute the hmac
        let mut digest = vec![];
        match compute_hmac(&data).await {
            Ok(data) => digest.extend(data.iter()),
            Err(e) => {
                // How to handle this error? It's more an
                // internal server error rather than a bad
                // request. Log the error the return 400.
                error!("couldn't compute hmac - {}", e.to_string());
                return Ok(Response::new(StatusCode::BadRequest));
            }
        }

        // Write the response
        info!("request from [{:?}] sucess", req.peer_addr());
        let mut res = Response::new(StatusCode::Ok);
        res.set_body(&digest[..]);

        Ok(res)
    })
    .await?;

    Ok(())
}

fn get_addr() -> String {
    for (k, v) in env::vars() {
        if k == "CRYPTOSERVER_BIND" {
            return v;
        }
    }

    DEFAULT_BIND.to_string()
}

async fn compute_hmac(data: &[u8]) -> Result<Vec<u8>> {
    let mut secret = [0; 32];
    get_secret(data, &mut secret).await?;
    let mut hasher = HmacSha256::new_varkey(&secret).expect("couldn't construct hasher");
    hasher.update(data);
    Ok(hasher.finalize().into_bytes().to_vec())
}

async fn get_secret(data: &[u8], buf: &mut [u8; 32]) -> Result<()> {
    let key_id = murmurhash3(data);

    match *MODE {
        Mode::Mode0 => get_secret0(buf).await,
        Mode::Mode16 => get_secret16(key_id, buf).await,
        Mode::Mode32 => get_secret32(key_id, buf).await,
    }
}

async fn get_secret0(buf: &mut [u8; 32]) -> Result<()> {
    let mut secret_file = SECRET_DIR.clone();
    secret_file.push("secret");

    OpenOptions::new()
        .read(true)
        .open(secret_file.to_str().unwrap())
        .await?
        .read_exact(&mut buf[..])
        .await?;

    Ok(())
}

async fn get_secret16(key_id: u32, buf: &mut [u8; 32]) -> Result<()> {
    let mut secret_file = SECRET_DIR.clone();
    secret_file.push("0000");

    // Zero the upper 16 bits
    let key_id = key_id & 0xffff;

    debug!("using secret {}", key_id);

    let mut file = OpenOptions::new()
        .read(true)
        .open(secret_file.to_str().unwrap())
        .await?;
    file.seek(SeekFrom::Start(key_id as u64 * 32)).await?;

    file.read_exact(&mut buf[..]).await?;

    Ok(())
}

async fn get_secret32(key_id: u32, buf: &mut [u8; 32]) -> Result<()> {
    let mut secret_file = SECRET_DIR.clone();

    // The filename is upper 16 bits
    let fname = format!("{:x}", (key_id & 0xffff0000) >> 16);
    debug!("using secret file {}", fname);

    secret_file.push(fname);

    // The secret is the lower 16 bits
    let key_id = key_id & 0xffff;

    let mut file = OpenOptions::new()
        .read(true)
        .open(secret_file.to_str().unwrap())
        .await?;
    file.seek(SeekFrom::Start(key_id as u64 * 32)).await?;

    file.read_exact(&mut buf[..]).await?;

    Ok(())
}
