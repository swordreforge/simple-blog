use std::path::PathBuf;
use gm_quic::QuicServer;
use bytes::{Buf, Bytes};
use reqwest::Client;

/// HTTP/3 帧类型
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u64)]
enum FrameType {
    Data = 0x0,
    Headers = 0x1,
    Priority = 0x2,
    CancelPush = 0x3,
    Settings = 0x4,
    PushPromise = 0x5,
    GoAway = 0x7,
    MaxPushId = 0xd,
    DuplicatePush = 0xe,
}

impl TryFrom<u64> for FrameType {
    type Error = u64;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0x0 => Ok(FrameType::Data),
            0x1 => Ok(FrameType::Headers),
            0x2 => Ok(FrameType::Priority),
            0x3 => Ok(FrameType::CancelPush),
            0x4 => Ok(FrameType::Settings),
            0x5 => Ok(FrameType::PushPromise),
            0x7 => Ok(FrameType::GoAway),
            0xd => Ok(FrameType::MaxPushId),
            0xe => Ok(FrameType::DuplicatePush),
            _ => Err(value),
        }
    }
}

/// HTTP/3 帧头部
struct FrameHeader {
    frame_type: FrameType,
    length: u64,
}

/// HTTP/3 Settings 参数
#[derive(Debug)]
struct SettingsFrame {
    max_header_list_size: Option<u64>,
    max_table_capacity: Option<u64>,
    blocked_streams: Option<u64>,
    enable_qpack: bool,
}

/// HTTP/3 头部帧
#[derive(Debug)]
struct HeadersFrame {
    header_block: Bytes,
    headers: Vec<(String, String)>,
}

/// HTTP/3 数据帧
struct DataFrame {
    data: Bytes,
}

/// HTTP 请求信息
struct HttpRequest {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Bytes,
}

/// QPACK 简化头部解码器
struct QpackDecoder {
    _max_table_capacity: u64,
}

impl QpackDecoder {
    fn new() -> Self {
        Self {
            _max_table_capacity: 4096,
        }
    }

    fn decode(&self, encoded: &Bytes) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        let mut headers = Vec::new();
        let mut pos = 0;
        let data = encoded.as_ref();

        while pos < data.len() {
            if pos + 1 > data.len() {
                break;
            }
            
            let first_byte = data[pos];
            pos += 1;
            
            if first_byte & 0x80 != 0 {
                if pos >= data.len() { break; }
                let _index = Self::read_varint(data, &mut pos)?;
            } else {
                let mut name = String::new();
                if first_byte & 0x40 != 0 {
                    let _name_index = Self::read_varint(data, &mut pos)?;
                } else {
                    let name_len = Self::read_varint(data, &mut pos)? as usize;
                    if pos + name_len > data.len() { break; }
                    name = String::from_utf8_lossy(&data[pos..pos + name_len]).to_string();
                    pos += name_len;
                }
                
                let value_len = Self::read_varint(data, &mut pos)? as usize;
                if pos + value_len > data.len() { break; }
                let value = String::from_utf8_lossy(&data[pos..pos + value_len]).to_string();
                pos += value_len;
                
                headers.push((name, value));
            }
        }
        
        Ok(headers)
    }

    fn read_varint(data: &[u8], pos: &mut usize) -> Result<u64, Box<dyn std::error::Error>> {
        if *pos >= data.len() {
            return Err("Unexpected end of data".into());
        }
        
        let first = data[*pos];
        *pos += 1;
        
        let prefix_len = 2;
        let mask = (1 << prefix_len) - 1;
        let mut value = (first & mask) as u64;
        
        if (value as u8) < mask {
            return Ok(value);
        }
        
        let mut m = 0;
        loop {
            if *pos >= data.len() {
                return Err("Unexpected end of data".into());
            }
            
            let b = data[*pos];
            *pos += 1;
            
            value += ((b & 0x7f) as u64) << m;
            m += 7;
            
            if b & 0x80 == 0 {
                break;
            }
        }
        
        Ok(value)
    }
}

impl Default for QpackDecoder {
    fn default() -> Self {
        Self::new()
    }
}

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

    let bind_addr = config.bind_addr.parse::<std::net::SocketAddr>()?;
    let cert_path = PathBuf::from(&config.cert_path);
    let key_path = PathBuf::from(&config.key_path);

    let quic_server = QuicServer::builder()
        .without_client_cert_verifier()
        .with_single_cert(cert_path.as_path(), key_path.as_path())
        .with_alpns([vec![b'h', b'3'], vec![b'h', b'3', b'-', b'2', b'9']])
        .listen(bind_addr)?;

    println!("✅ HTTP/3 服务器启动成功 (gm-quic)");

    accept_connections(quic_server, config.forward_addr).await
}

/// 接受并处理连接
async fn accept_connections(
    quic_server: std::sync::Arc<QuicServer>,
    forward_addr: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let http_client = Client::new();
    
    loop {
        match quic_server.accept().await {
            Ok((connection, pathway)) => {
                println!("🔗 新的 QUIC 连接");
                println!("🛣️  路径: {:?}", pathway);

                let forward_addr_clone = forward_addr.clone();
                let http_client_clone = http_client.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(connection, forward_addr_clone, http_client_clone).await {
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
    forward_addr: String,
    http_client: Client,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 处理 QUIC 连接");

    connection.handshaked().await;
    println!("✅ QUIC 握手完成");

    let qpack = QpackDecoder::new();

    loop {
        match connection.accept_bi_stream().await {
            Ok(Some((stream_id, (mut reader, writer)))) => {
                println!("📥 接收到流: {:?}", stream_id);

                let mut buffer = Vec::new();
                use tokio::io::AsyncReadExt;
                let _ = reader.read_to_end(&mut buffer).await;

                if !buffer.is_empty() {
                    // 解析 HTTP/3 帧并转发请求
                    if let Err(e) = process_and_forward(Bytes::from(buffer), &qpack, &forward_addr, &http_client, writer).await {
                        eprintln!("❌ 处理转发错误: {}", e);
                    }
                }
            }
            Ok(None) => {
                println!("🔌 连接结束");
                break;
            }
            Err(e) => {
                eprintln!("❌ 接受流错误: {}", e);
                break;
            }
        }
    }

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

/// 处理 HTTP/3 帧并转发请求
async fn process_and_forward(
    data: Bytes,
    qpack: &QpackDecoder,
    forward_addr: &str,
    http_client: &Client,
    mut writer: qconnection::StreamWriter,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut pos = 0;
    let data_slice = data.as_ref();
    let mut http_request = HttpRequest {
        method: "GET".to_string(),
        path: "/".to_string(),
        headers: Vec::new(),
        body: Bytes::new(),
    };

    while pos < data_slice.len() {
        let header = parse_frame_header(data_slice, &mut pos)?;
        
        if pos + header.length as usize > data_slice.len() {
            break;
        }
        
        let frame_data = &data_slice[pos..pos + header.length as usize];
        
        match header.frame_type {
            FrameType::Headers => {
                let headers_frame = parse_headers_frame(frame_data, qpack)?;
                http_request.headers = headers_frame.headers.clone();
                // 从头部提取方法和路径
                for (name, value) in &headers_frame.headers {
                    if name == ":method" {
                        http_request.method = value.clone();
                    } else if name == ":path" {
                        http_request.path = value.clone();
                    }
                }
            }
            FrameType::Data => {
                let data_frame = parse_data_frame(frame_data)?;
                http_request.body = data_frame.data.clone();
            }
            _ => {}
        }
        
        pos += header.length as usize;
    }

    // 转发到 Actix Web
    let url = format!("{}{}", forward_addr, http_request.path);
    println!("➡️  转发请求: {} {}", http_request.method, url);

    let mut req_builder = http_client.request(
        http_request.method.parse()?,
        &url
    );

    // 转发头部（跳过 HTTP/3 伪头部）
    for (name, value) in &http_request.headers {
        if !name.starts_with(':') {
            req_builder = req_builder.header(name, value);
        }
    }

    // 发送请求体
    if !http_request.body.is_empty() {
        req_builder = req_builder.body(http_request.body.to_vec());
    }

    let response = req_builder.send().await?;
    let status = response.status();
    let response_body = response.bytes().await?;

    println!("⬅️  响应状态: {}", status);
    println!("⬅️  响应体: {} bytes", response_body.len());

    // 发送 HTTP/3 响应（简化版：直接发送数据）
    // 实际实现需要构建 HTTP/3 帧格式（Headers + Data 帧）
    use tokio::io::AsyncWriteExt;
    let _ = writer.write_all(&response_body).await;
    let _ = writer.shutdown().await;

    Ok(())
}

fn read_varint(data: &[u8], pos: &mut usize) -> Result<u64, Box<dyn std::error::Error>> {
    if *pos >= data.len() {
        return Err("Unexpected end of data".into());
    }
    
    let first = data[*pos];
    *pos += 1;
    
    let prefix_len = 2;
    let mask = (1 << prefix_len) - 1;
    let mut value = (first & mask) as u64;
    
    if (value as u8) < mask {
        return Ok(value);
    }
    
    let mut m = 0;
    loop {
        if *pos >= data.len() {
            return Err("Unexpected end of data".into());
        }
        
        let b = data[*pos];
        *pos += 1;
        
        value += ((b & 0x7f) as u64) << m;
        m += 7;
        
        if b & 0x80 == 0 {
            break;
        }
    }
    
    Ok(value)
}

fn parse_frame_header(data: &[u8], pos: &mut usize) -> Result<FrameHeader, Box<dyn std::error::Error>> {
    let frame_type = read_varint(data, pos)?;
    let length = read_varint(data, pos)?;
    
    let frame_type = FrameType::try_from(frame_type)
        .map_err(|_| format!("Unknown frame type: {}", frame_type))?;
    
    Ok(FrameHeader { frame_type, length })
}

fn parse_settings_frame(data: &[u8]) -> Result<SettingsFrame, Box<dyn std::error::Error>> {
    let mut pos = 0;
    let mut settings = SettingsFrame {
        max_header_list_size: None,
        max_table_capacity: None,
        blocked_streams: None,
        enable_qpack: false,
    };
    
    while pos + 2 <= data.len() {
        let identifier = read_varint(data, &mut pos)?;
        let value = read_varint(data, &mut pos)?;
        
        match identifier {
            0x6 => settings.max_header_list_size = Some(value),
            0x1 => settings.max_table_capacity = Some(value),
            0x7 => settings.blocked_streams = Some(value),
            0x8 => settings.enable_qpack = value != 0,
            _ => println!("⚠️  未知设置: 0x{:x} = {}", identifier, value),
        }
    }
    
    Ok(settings)
}

fn parse_headers_frame(data: &[u8], qpack: &QpackDecoder) -> Result<HeadersFrame, Box<dyn std::error::Error>> {
    let header_block = Bytes::copy_from_slice(data);
    let headers = qpack.decode(&header_block)?;
    
    Ok(HeadersFrame { header_block, headers })
}

fn parse_data_frame(data: &[u8]) -> Result<DataFrame, Box<dyn std::error::Error>> {
    Ok(DataFrame {
        data: Bytes::copy_from_slice(data),
    })
}