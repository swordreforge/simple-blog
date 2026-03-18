use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RouteType {
    Memory,
    File,
    Database,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicRoute {
    pub id: Option<i64>,
    pub route_name: Option<String>,
    pub route_type: RouteType,
    pub path: String,
}

fn main() {
    // 测试 RouteType 序列化
    let memory = RouteType::Memory;
    let file = RouteType::File;
    let database = RouteType::Database;

    println!("RouteType 序列化测试:");
    println!("Memory: {}", serde_json::to_string(&memory).unwrap());
    println!("File: {}", serde_json::to_string(&file).unwrap());
    println!("Database: {}", serde_json::to_string(&database).unwrap());

    // 测试 DynamicRoute 序列化
    let route = DynamicRoute {
        id: Some(1),
        route_name: Some("Test Route".to_string()),
        route_type: RouteType::Memory,
        path: "/test".to_string(),
    };

    println!("\nDynamicRoute 序列化测试:");
    let json = serde_json::to_string_pretty(&route).unwrap();
    println!("{}", json);

    // 测试反序列化
    let deserialized: DynamicRoute = serde_json::from_str(&json).unwrap();
    println!("\n反序列化后的 route_type: {:?}", deserialized.route_type);
}
