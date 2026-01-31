use std::sync::Arc;
use quinn::{Endpoint, ServerConfig, crypto::rustls::QuicServerConfig};
use rustls::pki_types::CertificateDer;
use rustls::ServerConfig as RustlsServerConfig;
use rustls_pemfile::{certs, private_key};
use std::time::Duration;
use tokio::select;

/// HTTP/3 服务器配置
pub struct Http3ServerConfig {
    pub cert_path: String,
    pub key_path: String,
    pub bind_addr: String,
}

impl Default for Http3ServerConfig {
    fn default() -> Self {
        Self {
            cert_path: "cert.pem".to_string(),
            key_path: "key.pem".to_string(),
            bind_addr: "[::]:443".to_string(),
        }
    }
}

/// 启动 HTTP/3 服务器
pub async fn start_http3_server(config: Http3ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 启动 HTTP/3 服务器...");
    println!("📡 监听地址: {}", config.bind_addr);
    println!("🔒 证书文件: {}", config.cert_path);
    println!("🔑 私钥文件: {}", config.key_path);

    // 加载证书和私钥
    let cert_file = std::fs::read(&config.cert_path)?;
    let key_file = std::fs::read(&config.key_path)?;

    let cert_chain: Vec<CertificateDer<'static>> = certs(&mut &cert_file[..])
        .map(|result| result.map(|cert| CertificateDer::from(cert.to_vec())))
        .collect::<Result<Vec<_>, _>>()?;

    let key_der = private_key(&mut &key_file[..])?
        .ok_or("No private key found")?
        .try_into()?;

    // 创建 TLS 配置
    let mut tls_config = RustlsServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key_der)?;

    // 配置 ALPN 协议（HTTP/3 必须配置）
    tls_config.alpn_protocols = vec![b"h3".to_vec(), b"h3-29".to_vec()];

    // 创建 QUIC 服务器配置
    let mut server_config = ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(tls_config)?));

    // 配置传输参数
    let mut transport = quinn::TransportConfig::default();
    transport.max_concurrent_bidi_streams(100_u32.into());
    transport.max_idle_timeout(Some(Duration::from_secs(300).try_into()?));
    server_config.transport_config(Arc::new(transport));

    // 创建 QUIC endpoint
    let endpoint = Endpoint::server(server_config, config.bind_addr.parse()?)?;
    println!("✅ HTTP/3 服务器启动成功 (使用 QUIC 协议)");

    // 接受连接并处理请求
    accept_connections(endpoint).await
}

/// 接受并处理连接
async fn accept_connections(endpoint: Endpoint) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        select! {
            // 接受新的 QUIC 连接
            accept_result = endpoint.accept() => {
                if let Some(conn) = accept_result {
                    let conn = conn.await?;
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(conn).await {
                            eprintln!("❌ HTTP/3 连接处理错误: {}", e);
                        }
                    });
                }
            }
            // 检查服务器是否需要关闭
            _ = tokio::signal::ctrl_c() => {
                println!("🛑 收到停止信号，正在关闭 HTTP/3 服务器...");
                endpoint.close(0u32.into(), b"Server shutdown");
                break;
            }
        }
    }
    Ok(())
}

/// 处理单个连接
async fn handle_connection(
    conn: quinn::Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 新的 QUIC 连接: {}", conn.remote_address());

    // 简单的连接处理：保持连接打开
    tokio::select! {
        _ = conn.closed() => {
            println!("🔌 QUIC 连接关闭");
            Ok(())
        }
        _ = tokio::signal::ctrl_c() => {
            conn.close(0u32.into(), b"Server shutdown");
            Ok(())
        }
    }
}