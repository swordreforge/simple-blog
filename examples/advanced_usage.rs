//! 高级功能使用示例
//!
//! 展示如何使用数据库存储、版本控制、路由验证等高级功能。

#[cfg(all(feature = "database", feature = "sqlite"))]

use dynamic_route_actix::RouteEntry;
use dynamic_route_actix::core::{RouteRegistry, SerializableRoute, SimpleRoute, RouteValidator, RouteTypeMetadata};
use dynamic_route_actix::storage::{DatabaseStorage, DatabaseStorageConfig, DatabaseType, RouteStorage};
use actix_web::{HttpRequest, HttpResponse};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 动态路由库高级功能演示 ===\n");

    // 演示 1: 数据库存储和版本控制
    demo_database_storage().await?;

    // 演示 2: 路由验证
    demo_route_validation()?;

    // 演示 3: 自定义路由类型与版本控制
    demo_custom_route_with_versioning().await?;

    println!("\n=== 演示完成 ===");
    Ok(())
}

/// 演示 1: 数据库存储和版本控制
async fn demo_database_storage() -> Result<(), Box<dyn std::error::Error>> {
    println!("演示 1: 数据库存储和版本控制");
    println!("--------------------------------");

    // 创建 SQLite 数据库存储
    let config = DatabaseStorageConfig {
        database_type: DatabaseType::SQLite,
        database_url: "sqlite:routes_demo.db".to_string(),
        max_connections: 5,
        enable_versioning: true,
        max_versions: Some(5),
    };

    let storage = DatabaseStorage::new(config).await?;
    println!("✓ 创建了 SQLite 数据库存储");

    // 创建并保存初始路由
    let mut routes = HashMap::new();
    routes.insert(
        "/hello".to_string(),
        Box::new(SimpleRoute::new("Hello, World!", "text/plain")) as Box<dyn RouteEntry>,
    );
    routes.insert(
        "/api/status".to_string(),
        Box::new(SimpleRoute::new(r#"{"status":"ok"}"#, "application/json")) as Box<dyn RouteEntry>,
    );

    if let Err(e) = storage.save(&routes).await {
        println!("✗ 保存失败: {}", e);
        return Err(e.into());
    }
    println!("✓ 保存了初始路由: {}", routes.len());

    // 加载路由
    match storage.load().await {
        Ok(loaded_routes) => {
            println!("✓ 加载了路由: {}", loaded_routes.len());
        }
        Err(e) => {
            println!("✗ 加载失败: {}", e);
            return Err(e.into());
        }
    }

    // 更新路由（创建新版本）
    routes.insert(
        "/hello".to_string(),
        Box::new(SimpleRoute::new("Hello, Updated World!", "text/plain")) as Box<dyn RouteEntry>,
    );

    if let Err(e) = storage.save(&routes).await {
        println!("✗ 更新失败: {}", e);
        return Err(e.into());
    }
    println!("✓ 更新了 /hello 路由");

    // 获取路由版本历史
    match storage.get_route_versions("/hello").await {
        Ok(versions) => {
            println!("✓ 路由 /hello 有 {} 个版本", versions.len());
            for (i, version) in versions.iter().enumerate() {
                println!("  版本 {}: {} (创建于: {})", i + 1, version.body, version.created_at);
            }
        }
        Err(e) => {
            println!("✗ 获取版本历史失败: {}", e);
        }
    }

    // 清理
    std::fs::remove_file("routes_demo.db").ok();
    println!("✓ 清理了数据库文件\n");

    Ok(())
}

/// 演示 2: 路由验证
fn demo_route_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("演示 2: 路由验证");
    println!("--------------------------------");

    // 创建验证器
    let validator = RouteValidator::with_defaults();
    println!("✓ 创建了路由验证器");

    // 验证有效路由
    let valid_route = SerializableRoute {
        route_type: "SimpleRoute".to_string(),
        body: "Hello".to_string(),
        content_type: "text/plain".to_string(),
        extra_data: None,
    };

    match validator.validate_route("/test", &valid_route) {
        Ok(_) => println!("✓ 路由验证通过"),
        Err(e) => println!("✗ 路由验证失败: {}", e),
    }

    // 验证无效路由（缺少前导 /）
    let invalid_route = SerializableRoute {
        route_type: "SimpleRoute".to_string(),
        body: "Hello".to_string(),
        content_type: "text/plain".to_string(),
        extra_data: None,
    };

    match validator.validate_route("test", &invalid_route) {
        Ok(_) => println!("✗ 不应该验证通过"),
        Err(e) => println!("✓ 正确拒绝了无效路径: {}", e),
    }

    println!();

    Ok(())
}

/// 演示 3: 自定义路由类型与版本控制
async fn demo_custom_route_with_versioning() -> Result<(), Box<dyn std::error::Error>> {
    println!("演示 3: 自定义路由类型与版本控制");
    println!("--------------------------------");

    // 定义自定义路由类型
    #[derive(Debug, Clone)]
    struct TimedRoute {
        body: String,
        content_type: String,
        timeout_ms: u64,
    }

    impl TimedRoute {
        fn new(body: &str, content_type: &str, timeout_ms: u64) -> Self {
            Self {
                body: body.to_string(),
                content_type: content_type.to_string(),
                timeout_ms,
            }
        }
    }

    impl RouteEntry for TimedRoute {
        fn handle(&self, _req: &HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> {
            let body = self.body.clone();
            let content_type = self.content_type.clone();
            Box::pin(async move {
                HttpResponse::Ok().content_type(content_type).body(body)
            })
        }

        fn clone_box(&self) -> Box<dyn RouteEntry> {
            Box::new(self.clone())
        }

        fn to_serializable(&self) -> SerializableRoute {
            let extra_data = serde_json::json!({
                "timeout_ms": self.timeout_ms
            }).to_string();

            SerializableRoute {
                route_type: "TimedRoute".to_string(),
                body: self.body.clone(),
                content_type: self.content_type.clone(),
                extra_data: Some(extra_data),
            }
        }

        fn from_serializable(data: SerializableRoute) -> Box<dyn RouteEntry>
        where
            Self: Sized,
        {
            let timeout_ms = if let Some(ref extra) = data.extra_data {
                serde_json::from_str(extra).unwrap_or(1000)
            } else {
                1000
            };

            Box::new(TimedRoute::new(&data.body, &data.content_type, timeout_ms))
        }
    }

    // 注册自定义路由类型
    if let Err(e) = RouteRegistry::register("TimedRoute", TimedRoute::from_serializable) {
        println!("✗ 注册失败: {}", e);
        return Err(e.into());
    }
    println!("✓ 注册了 TimedRoute 类型");

    // 创建数据库存储
    let config = DatabaseStorageConfig {
        database_type: DatabaseType::SQLite,
        database_url: "sqlite:timed_routes.db".to_string(),
        enable_versioning: true,
        ..Default::default()
    };

    let storage = DatabaseStorage::new(config).await?;
    println!("✓ 创建了数据库存储");

    // 创建并保存自定义路由
    let mut routes = HashMap::new();
    routes.insert(
        "/timed".to_string(),
        Box::new(TimedRoute::new("Response with 1s timeout", "text/plain", 1000)) as Box<dyn RouteEntry>,
    );

    if let Err(e) = storage.save(&routes).await {
        println!("✗ 保存失败: {}", e);
        return Err(e.into());
    }
    println!("✓ 保存了自定义路由");

    // 加载并验证
    match storage.load().await {
        Ok(loaded_routes) => {
            if let Some(route) = loaded_routes.get("/timed") {
                let serializable = route.to_serializable();
                println!("✓ 加载的路由类型: {}", serializable.route_type);
                if let Some(ref extra) = serializable.extra_data {
                    println!("✓ 额外数据: {}", extra);
                }
            }
        }
        Err(e) => {
            println!("✗ 加载失败: {}", e);
        }
    }

    // 清理
    std::fs::remove_file("timed_routes.db").ok();
    RouteRegistry::unregister("TimedRoute");
    println!("✓ 清理了资源和注册表\n");

    Ok(())
}