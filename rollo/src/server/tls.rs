use rustls_pemfile::{certs, rsa_private_keys};
use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};

pub(crate) fn load_config(certificate: &Path, key: &Path) -> io::Result<ServerConfig> {
    let certs = certs(&mut BufReader::new(File::open(certificate)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
        .map(|mut certs| certs.drain(..).map(Certificate).collect())?;

    let mut keys: Vec<PrivateKey> = rsa_private_keys(&mut BufReader::new(File::open(key)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
        .map(|mut keys| keys.drain(..).map(PrivateKey).collect())?;

    let server_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, keys.swap_remove(0))
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Problem when setting cert to config",
            )
        })?;

    Ok(server_config)
}
