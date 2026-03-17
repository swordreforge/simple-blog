//! 自定义路由类型示例
//!
//! 演示如何创建自定义路由类型并注册到路由注册表中。

use actix_web::{HttpRequest, HttpResponse};
use dynamic_route_actix::core::RouteRegistry;
use dynamic_route_actix::{RouteEntry, SerializableRoute};
use std::future::Future;
use std::pin::Pin;

/// 带超时的路由
///
/// 演示如何扩展 SerializableRoute 来存储额外的自定义数据。
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
            // 在实际应用中，这里可以实现超时逻辑
            HttpResponse::Ok().content_type(content_type).body(body)
        })
    }

    fn clone_box(&self) -> Box<dyn RouteEntry> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn to_serializable(&self) -> SerializableRoute {
        // 使用 extra_data 存储自定义字段
        let extra_data = serde_json::json!({
            "timeout_ms": self.timeout_ms
        })
        .to_string();

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
        // 从 extra_data 解析自定义字段
        let timeout_ms = if let Some(ref extra) = data.extra_data {
            serde_json::from_str(extra).unwrap_or(1000)
        } else {
            1000
        };

        Box::new(TimedRoute::new(&data.body, &data.content_type, timeout_ms))
    }
}

/// 带响应头的路由
///
/// 演示如何在自定义路由中添加 HTTP 响应头。
#[derive(Debug, Clone)]
struct HeaderRoute {
    body: String,
    headers: Vec<(String, String)>,
}

impl HeaderRoute {
    fn new(body: &str, headers: Vec<(&str, &str)>) -> Self {
        Self {
            body: body.to_string(),
            headers: headers
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}

impl RouteEntry for HeaderRoute {
    fn handle(&self, _req: &HttpRequest) -> Pin<Box<dyn Future<Output = HttpResponse> + Send>> {
        let body = self.body.clone();
        let headers = self.headers.clone();
        Box::pin(async move {
            let mut builder = HttpResponse::Ok();
            for (key, value) in headers {
                builder.append_header((key.as_str(), value.as_str()));
            }
            builder.body(body)
        })
    }

    fn clone_box(&self) -> Box<dyn RouteEntry> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn to_serializable(&self) -> SerializableRoute {
        // 使用 extra_data 存储自定义字段
        let extra_data = serde_json::json!({
            "headers": self.headers
        })
        .to_string();

        SerializableRoute {
            route_type: "HeaderRoute".to_string(),
            body: self.body.clone(),
            content_type: "text/plain".to_string(),
            extra_data: Some(extra_data),
        }
    }

    fn from_serializable(data: SerializableRoute) -> Box<dyn RouteEntry>
    where
        Self: Sized,
    {
        // 从 extra_data 解析自定义字段
        #[derive(serde::Deserialize)]
        struct HeaderData {
            headers: Vec<(String, String)>,
        }

        let headers = if let Some(ref extra) = data.extra_data {
            serde_json::from_str::<HeaderData>(extra)
                .map(|h| h.headers)
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        Box::new(HeaderRoute {
            body: data.body,
            headers,
        })
    }
}

fn main() {
    println!("=== 自定义路由类型示例 ===\n");

    // 注册自定义路由类型
    println!("1. 注册自定义路由类型");
    RouteRegistry::register("TimedRoute", TimedRoute::from_serializable)
        .expect("Failed to register TimedRoute");
    RouteRegistry::register("HeaderRoute", HeaderRoute::from_serializable)
        .expect("Failed to register HeaderRoute");
    println!("   ✓ TimedRoute 已注册");
    println!("   ✓ HeaderRoute 已注册\n");

    // 检查注册的类型
    println!("2. 检查已注册的路由类型:");
    let types = RouteRegistry::list_types();
    for route_type in &types {
        println!("   - {}", route_type);
    }
    println!();

    // 测试 TimedRoute 序列化和反序列化
    println!("3. 测试 TimedRoute:");
    let timed_route = TimedRoute::new("Hello with timeout!", "text/plain", 5000);
    let serializable = timed_route.to_serializable();
    println!("   序列化数据: {:?}", serializable);

    let restored_route = RouteRegistry::create_route(serializable.clone()).unwrap();
    println!("   反序列化成功: {:?}", restored_route.to_serializable());
    println!();

    // 测试 HeaderRoute 序列化和反序列化
    println!("4. 测试 HeaderRoute:");
    let header_route = HeaderRoute::new(
        "Hello with headers!",
        vec![
            ("X-Custom-Header", "custom-value"),
            ("X-Request-ID", "12345"),
        ],
    );
    let serializable = header_route.to_serializable();
    println!("   序列化数据: {:?}", serializable);

    let restored_route = RouteRegistry::create_route(serializable.clone()).unwrap();
    println!("   反序列化成功: {:?}", restored_route.to_serializable());
    println!();

    // 测试未注册的类型
    println!("5. 测试未注册的类型:");
    let unknown_data = SerializableRoute {
        route_type: "UnknownRoute".to_string(),
        body: "Test".to_string(),
        content_type: "text/plain".to_string(),
        extra_data: None,
    };
    match RouteRegistry::create_route(unknown_data) {
        Ok(_) => println!("   意外成功！"),
        Err(e) => println!("   预期的错误: {}", e),
    }
    println!();

    // 清理
    println!("6. 清理自定义路由类型:");
    RouteRegistry::unregister("TimedRoute");
    RouteRegistry::unregister("HeaderRoute");
    println!("   ✓ 自定义路由类型已注销");

    println!("\n=== 示例完成 ===");
}
