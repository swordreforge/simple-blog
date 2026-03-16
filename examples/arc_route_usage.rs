//! ArcRouteEntry 使用示例
//!
//! 本示例展示如何使用 ArcRouteEntry 包装器来优化路由条目的性能。

use dynamic_route_actix::{ArcRouteEntry, RouteEntry, RouteTable, SimpleRoute};

fn main() {
    println!("=== ArcRouteEntry 使用示例 ===\n");

    // 示例 1: 创建和包装路由
    println!("1. 创建 SimpleRoute 并使用 ArcRouteEntry 包装");
    let simple_route = SimpleRoute::new("Hello, World!", "text/plain");
    let arc_route = ArcRouteEntry::new(simple_route);
    println!("   ✓ ArcRouteEntry 创建成功");
    println!("   ✓ 引用计数: {}", arc_route.ref_count());

    // 示例 2: 零成本克隆
    println!("\n2. 零成本克隆演示");
    let cloned_route = arc_route.clone_box();
    println!("   ✓ 克隆后引用计数: {}", arc_route.ref_count());
    println!("   ✓ 两个路由共享相同的数据");

    // 示例 3: 在路由表中使用
    println!("\n3. 在路由表中使用 ArcRouteEntry");
    let table = RouteTable::new();

    // 添加多个共享相同响应的路由
    let response_content = ArcRouteEntry::new(SimpleRoute::new("API Version 1.0", "application/json"));

    table.insert("/api/v1".into(), Box::new(response_content.clone()));
    table.insert("/api/v1/info".into(), Box::new(response_content.clone()));
    table.insert("/api/v1/status".into(), Box::new(response_content.clone()));

    println!("   ✓ 添加了 3 个共享数据的路由");
    println!("   ✓ 引用计数: {}", response_content.ref_count());

    // 示例 4: 序列化和反序列化
    println!("\n4. 序列化和反序列化");
    let original_route = ArcRouteEntry::new(SimpleRoute::new("Test Content", "text/html"));
    let serializable = original_route.to_serializable();
    println!("   ✓ 序列化成功");
    println!("   ✓ 路由类型: {}", serializable.route_type);
    println!("   ✓ 响应体: {}", serializable.body);

    // 示例 5: 从 Box<dyn RouteEntry> 创建
    println!("\n5. 从 Box<dyn RouteEntry> 创建 ArcRouteEntry");
    let boxed: Box<dyn RouteEntry> = Box::new(SimpleRoute::new("Boxed Route", "application/xml"));
    let arc_from_boxed = ArcRouteEntry::from_boxed(boxed);
    println!("   ✓ 从 Box 创建成功");
    println!("   ✓ 引用计数: {}", arc_from_boxed.ref_count());

    // 示例 6: 性能对比
    println!("\n6. 性能优势说明");
    println!("   普通路由克隆:");
    println!("   - 每次克隆复制整个字符串数据");
    println!("   - 内存开销: O(n)，n 为字符串长度");
    println!("   - 适合小规模路由");
    println!();
    println!("   ArcRouteEntry 克隆:");
    println!("   - 仅增加引用计数（原子操作）");
    println!("   - 内存开销: O(1)");
    println!("   - 适合大规模路由和高并发场景");
    println!("   - 多个路由可共享相同数据");

    // 示例 7: 实际应用场景
    println!("\n7. 实际应用场景");
    println!("   • API 网关: 多个端点返回相同的错误响应");
    println!("   • 微服务: 多个服务共享通用响应模板");
    println!("   • CDN: 缓存相同内容的多个路由");
    println!("   • 负载均衡: 多个后端共享相同的路由配置");

    println!("\n=== 示例完成 ===");
}
