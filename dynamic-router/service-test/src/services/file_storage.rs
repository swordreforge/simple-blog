use std::path::Path;
use dynamic_route_actix::{RouteTable, core::SimpleRoute};
use std::io;

/// 获取路由文件目录
pub fn get_routes_dir() -> io::Result<String> {
    let routes_dir = "./routes";
    if !Path::new(routes_dir).exists() {
        std::fs::create_dir_all(routes_dir)?;
    }
    Ok(routes_dir.to_string())
}

/// 将路径转换为安全的文件名
pub fn path_to_filename(path: &str) -> String {
    path.replace('/', "_")
        .replace('.', "_")
        .replace(' ', "_")
}

/// 保存路由到文件
pub fn save_route_to_file(path: &str, body: &str, content_type: &str) -> io::Result<()> {
    let routes_dir = get_routes_dir()?;
    let filename = path_to_filename(path);
    let file_path = format!("{}/{}.json", routes_dir, filename);
    
    let route_data = serde_json::json!({
        "path": path,
        "body": body,
        "content_type": content_type,
        "created_at": chrono::Utc::now().to_rfc3339()
    });
    
    std::fs::write(&file_path, serde_json::to_string_pretty(&route_data)?)?;
    Ok(())
}

/// 从文件中删除路由
pub fn delete_route_from_file(path: &str) -> io::Result<()> {
    let routes_dir = get_routes_dir()?;
    let filename = path_to_filename(path);
    let file_path = format!("{}/{}.json", routes_dir, filename);
    
    if Path::new(&file_path).exists() {
        std::fs::remove_file(&file_path)?;
    }
    Ok(())
}

/// 列出所有文件路由
pub fn list_file_routes() -> io::Result<Vec<String>> {
    let routes_dir = get_routes_dir()?;
    let mut routes = Vec::new();
    
    for entry in std::fs::read_dir(&routes_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(path_str) = data.get("path").and_then(|p| p.as_str()) {
                        routes.push(path_str.to_string());
                    }
                }
            }
        }
    }
    
    routes.sort();
    Ok(routes)
}

/// 清空所有文件路由
pub fn clear_file_routes() -> io::Result<()> {
    let routes_dir = get_routes_dir()?;
    
    for entry in std::fs::read_dir(&routes_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            std::fs::remove_file(&path)?;
        }
    }
    
    Ok(())
}

/// 从文件加载所有路由到内存
pub fn load_routes_from_file(route_table: &RouteTable) -> io::Result<()> {
    let routes_dir = get_routes_dir()?;
    
    for entry in std::fs::read_dir(&routes_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let (Some(path_str), Some(body), Some(content_type)) = (
                        data.get("path").and_then(|p| p.as_str()),
                        data.get("body").and_then(|b| b.as_str()),
                        data.get("content_type").and_then(|c| c.as_str())
                    ) {
                        let route = SimpleRoute::new(body, content_type);
                        route_table.insert(path_str.to_string(), Box::new(route));
                    }
                }
            }
        }
    }
    
    Ok(())
}