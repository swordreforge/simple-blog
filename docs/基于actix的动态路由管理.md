构建一个基于 Actix-Web 的动态路由库，核心目标是将我们之前讨论的“共享状态 + 万能路由”模式封装为可复用的组件，让使用者能够轻松地在 Actix 应用中实现运行时的路由增删改查。下面是一个建议的架构设计。

---

## 整体架构图

```text
┌─────────────────────────────────────────────┐
│                你的库 crate                  │
│  ┌─────────────┐   ┌─────────────────────┐  │
│  │ 核心模块     │   │ Actix 集成模块      │  │
│  │ - RouteTable │   │ - UniversalHandler │  │
│  │ - Entry trait│   │ - Admin endpoints  │  │
│  └─────────────┘   └─────────────────────┘  │
│         ↓                 ↓                  │
│  ┌─────────────────────────────────────┐    │
│  │       可选的持久化模块               │    │
│  │   (FileStorage / DatabaseStorage)   │    │
│  └─────────────────────────────────────┘    │
└─────────────────────────────────────────────┘
                        ↓
             依赖: actix-web, tokio, serde...
```

## 核心模块设计

### 1. 路由条目抽象 `RouteEntry`
允许用户自定义路由处理逻辑，库提供默认的字符串响应实现，但应允许扩展。
```rust
pub trait RouteEntry: Send + Sync + 'static {
    fn handle(&self, req: &HttpRequest) -> BoxFuture<'static, HttpResponse>;
}
```
默认实现：
```rust
pub struct SimpleRoute {
    body: String,
    content_type: String,
}
impl RouteEntry for SimpleRoute { ... }
```

### 2. 路由表 `RouteTable`
线程安全的容器，存储路径到 `RouteEntry` 的映射。内部使用 `RwLock<HashMap<String, Box<dyn RouteEntry>>>`。

```rust
pub struct RouteTable {
    inner: RwLock<HashMap<String, Box<dyn RouteEntry>>>,
}

impl RouteTable {
    pub fn new() -> Self { ... }
    pub fn insert(&self, path: String, entry: Box<dyn RouteEntry>) { ... }
    pub fn remove(&self, path: &str) -> bool { ... }
    pub fn get(&self, path: &str) -> Option<impl Deref<Target = dyn RouteEntry>> { ... }
}
```

### 3. 持久化 trait（可选）
允许将路由表持久化到文件或数据库，库提供默认实现，用户可自定义。
```rust
#[async_trait]
pub trait RouteStorage: Send + Sync {
    async fn load(&self) -> Result<HashMap<String, Box<dyn RouteEntry>>>;
    async fn save(&self, routes: &HashMap<String, Box<dyn RouteEntry>>) -> Result<()>;
}
```

## Actix 集成模块

### 1. 万能路由处理器
一个 Actix handler，从 `RouteTable` 中查找路径并执行对应 `RouteEntry`。
```rust
pub async fn universal_handler(
    req: HttpRequest,
    table: web::Data<RouteTable>,
) -> HttpResponse {
    let path = req.match_info().query("tail");
    if let Some(entry) = table.get(path) {
        entry.handle(&req).await
    } else {
        HttpResponse::NotFound().body("Not found")
    }
}
```

### 2. 管理端点（可选）
提供一组预设的管理 API，允许通过 HTTP 动态增删路由。用户可以选择是否挂载到自己的 App 中。
```rust
pub fn admin_routes() -> actix_web::Scope {
    web::scope("/admin/routes")
        .route("", web::post().to(add_route_handler))
        .route("/{path}", web::delete().to(remove_route_handler))
        .route("", web::get().to(list_routes_handler))
}
```

### 3. 方便的应用构建辅助
提供一个扩展 trait，方便用户快速集成：
```rust
pub trait DynamicRouteApp {
    fn with_dynamic_routes(self, table: RouteTable) -> Self;
    fn with_admin_routes(self) -> Self;
}

impl DynamicRouteApp for App<...> { ... }
```

## 使用示例

```rust
use dynamic_route_actix::{RouteTable, SimpleRoute, admin_routes, universal_handler};

#[actix_web::main]
async fn main() {
    let table = RouteTable::new();
    table.insert("/hello".into(), SimpleRoute::new("world", "text/plain").into());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(table.clone()))
            .service(admin_routes())  // 挂载管理API
            .route("/{tail:.*}", web::get().to(universal_handler))
            .route("/{tail:.*}", web::post().to(universal_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## 设计要点总结

1. **线程安全**：`RouteTable` 内部使用 `RwLock`，可安全地在多个 Actix worker 间共享。
2. **可扩展性**：通过 `RouteEntry` trait，用户可以实现任意复杂的处理逻辑（如调用模板引擎、查询数据库）。
3. **与 Actix 深度集成**：提供 `web::Data` 注入、预置管理端点，用户几乎零配置即可获得完整功能。
4. **持久化可选**：通过 `RouteStorage` trait，支持启动时加载、变更时自动保存，用户可自行实现存储后端。
5. **性能**：读操作仅锁 `HashMap`，写操作（增删）加写锁，适合读多写少的场景。万能路由直接调用用户逻辑，无额外抽象开销。

## 依赖声明（Cargo.toml）

```toml
[dependencies]
actix-web = "4"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
async-trait = "0.1"
```

这样一个库既保持了 Actix 的高性能，又提供了灵活的运行时路由管理能力，与之前讨论的“用户上传文件、填变量、实时生效”的需求完美契合。
