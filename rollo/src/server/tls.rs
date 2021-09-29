use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};
//"../certificates/server.key.pem"
use rustls::internal::pemfile::{certs, rsa_private_keys};
use rustls::{NoClientAuth, ServerConfig};

pub(crate) fn load_config(certificate: &Path, key: &Path) -> io::Result<ServerConfig> {
    let certs = certs(&mut BufReader::new(File::open(certificate)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))?;

    let mut keys = rsa_private_keys(&mut BufReader::new(File::open(key)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))?;

    let mut server_config = ServerConfig::new(NoClientAuth::new());
    server_config
        .set_single_cert(certs, keys.swap_remove(0))
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Problem when setting cert to config",
            )
        })?;

    Ok(server_config)
}
