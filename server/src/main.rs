use anyhow::Result;
use std::time::Duration;
use tracing::error;
use tracing::info;
use tracing::info_span;
use tracing::Instrument;
use wtransport::endpoint::IncomingSession;
use wtransport::tls::CertificateChain;
use wtransport::tls::PrivateKey;
use wtransport::Endpoint;
use wtransport::Identity;
use wtransport::ServerConfig;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_level(true)
        .init();

    let new_cert = CertificateChain::load_pemfile("certificate.pem").await?;
    let new_key = PrivateKey::load_pemfile("certificate.key").await?;

    let identity = Identity::new(new_cert, new_key);

    let config = ServerConfig::builder()
        .with_bind_default(4433)
        .with_identity(identity)
        // .with_custom_tls(tls_config)
        .keep_alive_interval(Some(Duration::from_secs(3)))
        .build();

    let server = Endpoint::server(config)?;

    info!("Server ready!");
    info!("Listening on port 4433");

    for id in 0.. {
        let incoming_session = server.accept().await;
        tokio::spawn(handle_connection(incoming_session).instrument(info_span!("Connection", id)));
    }

    Ok(())
}

async fn handle_connection(incoming_session: IncomingSession) {
    let result = handle_connection_impl(incoming_session).await;
    error!("{:?}", result);
}

async fn handle_connection_impl(incoming_session: IncomingSession) -> Result<()> {
    let mut buffer = vec![0; 65536].into_boxed_slice();

    info!("Waiting for session request...");

    let session_request = incoming_session.await?;

    info!(
        "New session: Authority: '{}', Path: '{}'",
        session_request.authority(),
        session_request.path()
    );

    let connection = session_request.accept().await?;

    info!("Waiting for data from client...");

    let mut stream = connection.open_uni().await?.await?;
    tokio::spawn(async move {
        let mut counter = 1;
        loop {
            let message = format!("{}", counter);
            if let Err(e) = stream.write_all(message.as_bytes()).await {
                error!("Stream write error: {:?}", e);
                break;
            }
            info!("Sent (uni): {}", message);
            counter += 1;
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    loop {
        tokio::select! {
            stream = connection.accept_bi() => {
                let mut stream = stream?;
                info!("Accepted BI stream");

                let bytes_read = match stream.1.read(&mut buffer).await? {
                    Some(bytes_read) => bytes_read,
                    None => continue,
                };

                let str_data = std::str::from_utf8(&buffer[..bytes_read])?;

                info!("Received (bi) '{str_data}' from client");

                stream.0.write_all(b"ACK").await?;
            }
            stream = connection.accept_uni() => {
                let mut stream = stream?;
                info!("Accepted UNI stream");

                let bytes_read = match stream.read(&mut buffer).await? {
                    Some(bytes_read) => bytes_read,
                    None => continue,
                };

                let str_data = std::str::from_utf8(&buffer[..bytes_read])?;

                info!("Received (uni) '{str_data}' from client");

                let mut stream = connection.open_uni().await?.await?;
                stream.write_all(b"ACK").await?;
            }
            dgram = connection.receive_datagram() => {
                let dgram = dgram?;
                let str_data = std::str::from_utf8(&dgram)?;

                info!("Received (dgram) '{str_data}' from client");

                connection.send_datagram(b"ACK")?;
            }
        }
    }
}
