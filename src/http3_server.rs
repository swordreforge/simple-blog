use std::path::PathBuf;
use gm_quic::QuicServer;

/// HTTP/3 服务器配置
pub struct Http3ServerConfig {
    pub cert_path: String,
    pub key_path: String,
    pub bind_addr: String,
    pub forward_addr: String,
}

impl Default for Http3ServerConfig {
    fn default() -> Self {
        Self {
            cert_path: "cert.pem".to_string(),
            key_path: "key.pem".to_string(),
            bind_addr: "[::]:443".to_string(),
            forward_addr: "http://127.0.0.1:8080".to_string(),
        }
    }
}

/// 启动 HTTP/3 服务器（使用 gm-quic）
pub async fn start_http3_server(config: Http3ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 启动 HTTP/3 服务器（gm-quic 实现）...");
    println!("📡 监听地址: {}", config.bind_addr);
    println!("🔒 证书文件: {}", config.cert_path);
    println!("🔑 私钥文件: {}", config.key_path);
    println!("➡️  转发目标: {}", config.forward_addr);
    println!("⚠️  当前实现: QUIC 连接管理，HTTP/3 协议解析待实现");

    // 解析绑定地址
    let bind_addr = config.bind_addr.parse::<std::net::SocketAddr>()?;

    // 加载证书和私钥
    let cert_path = PathBuf::from(&config.cert_path);
    let key_path = PathBuf::from(&config.key_path);

    // 创建 QUIC 服务器
    let quic_server = QuicServer::builder()
        .without_client_cert_verifier()
        .with_single_cert(cert_path.as_path(), key_path.as_path())
        .with_alpns([vec![b'h', b'3'], vec![b'h', b'3', b'-', b'2', b'9']])
        .listen(bind_addr)?;

    println!("✅ HTTP/3 服务器启动成功 (gm-quic)");

    // 接受连接并处理请求
    accept_connections(quic_server).await
}

/// 接受并处理连接
async fn accept_connections(
    quic_server: std::sync::Arc<QuicServer>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        match quic_server.accept().await {
            Ok((connection, pathway)) => {
                println!("🔗 新的 QUIC 连接");
                println!("🛣️  路径: {:?}", pathway);

                let forward_addr = "http://127.0.0.1:8080".to_string();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(connection, forward_addr).await {
                        eprintln!("❌ HTTP/3 连接处理错误: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("❌ 接受连接错误: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
}

/// 处理单个连接
async fn handle_connection(
    connection: std::sync::Arc<qconnection::Connection>,
    _forward_addr: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 处理 QUIC 连接");

    // TODO: 完整的 HTTP/3 实现需要：
    // 1. 接受 QUIC 流
    // 2. 解析 HTTP/3 帧（HEADERS, DATA, SETTINGS 等）
    // 3. 使用 QPACK 解压缩头部
    // 4. 提取 HTTP 请求
    // 5. 转发到 Actix Web
    // 6. 返回响应

    // 简单的连接处理：保持连接打开
    tokio::select! {
        _ = connection.terminated() => {
            println!("🔌 QUIC 连接关闭");
            Ok(())
        }
        _ = tokio::signal::ctrl_c() => {
            connection.close("Server shutdown".into(), 0);
            Ok(())
        }
    }
}