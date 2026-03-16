use dynamic_route_actix::{RouteTable, SimpleRoute};

fn main() {
    // 创建路由表
    let table = RouteTable::new();

    println!("初始路由表状态:");
    println!("  路由数量: {}", table.count());
    println!("  路由列表: {:?}\n", table.list_paths());

    // 添加几个路由
    println!("添加路由:");

    table.insert(
        "/".into(),
        Box::new(SimpleRoute::new("欢迎使用动态路由系统", "text/plain")),
    );
    println!("  添加: /");

    table.insert(
        "/api".into(),
        Box::new(SimpleRoute::new(
            r#"{\"status\":\"ok\"}"#,
            "application/json",
        )),
    );
    println!("  添加: /api");

    table.insert(
        "/about".into(),
        Box::new(SimpleRoute::new("关于我们", "text/html")),
    );
    println!("  添加: /about");

    println!("\n添加后的路由表状态:");
    println!("  路由数量: {}", table.count());
    println!("  路由列表: {:?}\n", table.list_paths());

    // 查询路由
    println!("查询路由:");

    if table.contains("/api") {
        let result = table.get_with("/api", |_route| {
            // 在实际使用中，这里会调用 _route.handle(&req).await
            "路由存在"
        });
        println!("  /api: {}", result.unwrap());
    }

    if !table.contains("/nonexistent") {
        println!("  /nonexistent: 路由不存在");
    }

    // 移除路由
    println!("\n移除路由:");
    println!("  移除: /about");
    table.remove("/about");

    println!("\n移除后的路由表状态:");
    println!("  路由数量: {}", table.count());
    println!("  路由列表: {:?}\n", table.list_paths());

    // 测试覆盖
    println!("测试路由覆盖:");
    table.insert(
        "/api".into(),
        Box::new(SimpleRoute::new(
            r#"{\"status\":\"updated\"}"#,
            "application/json",
        )),
    );
    println!("  覆盖: /api");
    println!("  当前路由数量: {}", table.count());

    // 清空路由表
    println!("\n清空路由表:");
    table.clear();
    println!("  路由数量: {}", table.count());
    println!("  路由列表: {:?}", table.list_paths());
}
