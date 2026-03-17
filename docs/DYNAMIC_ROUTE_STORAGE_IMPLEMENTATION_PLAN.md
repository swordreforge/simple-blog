# 动态路由存储实现计划

## 当前状态分析

### 已完成的功能
1. ✅ 数据库基础架构（表创建、CRUD操作）
2. ✅ 动态路由路径规范化（解决双斜杠问题）
3. ✅ 静态路由冲突检测
4. ✅ 基本的路由管理API（创建、更新、删除、查询）
5. ✅ 路由测试接口

### 当前问题
当前系统只从数据库读取路由类型信息，但**未根据路由类型加载实际内容**：
- 数据库存储了路由元数据（路径、类型、配置等）
- 但没有实现根据存储类型（memory/file）加载路由内容的逻辑
- 内存类型应该直接从内存读取
- 文件类型应该从 `./routes` 目录加载自动生成的路由文件

## 存储类型设计

### 1. 内存存储（Memory Storage）
**特点：**
- 路由内容直接存储在数据库的 `handler_config` 字段中
- 适合简单的静态内容、重定向等
- 加载速度快，无需文件I/O

**使用场景：**
- 静态文本内容（HTML片段、JSON响应）
- 简单重定向（Redirect处理器）
- 基于模板的简单页面

**数据结构：**
```json
{
  "handler_type": "Static",
  "handler_config": {
    "content": "<h1>Hello World</h1>",
    "content_type": "text/html"
  }
}
```

### 2. 文件存储（File Storage）
**特点：**
- 路由内容存储在文件系统中
- 数据库只存储文件路径引用
- 支持热重载（文件修改后自动生效）
- 适合复杂的HTML页面、大型内容

**文件组织结构：**
```
./routes/
├── about.html
├── contact.html
├── products/
│   ├── product-1.html
│   └── product-2.html
└── custom/
    └── my-custom-page.html
```

**数据结构：**
```json
{
  "handler_type": "Template",
  "handler_config": {
    "file_path": "/routes/about.html",
    "template_name": "about"
  }
}
```

## 实现计划

### 阶段一：数据库架构扩展（1-2天）

#### 1.1 扩展 `dynamic_routes` 表
```sql
-- 添加存储类型字段
ALTER TABLE dynamic_routes ADD COLUMN storage_type TEXT NOT NULL DEFAULT 'memory';
-- storage_type 可选值: 'memory', 'file'

-- 添加文件路径字段（用于file类型）
ALTER TABLE dynamic_routes ADD COLUMN file_path TEXT;
-- 存储相对于 ./routes 目录的文件路径

-- 添加文件最后修改时间（用于热重载检测）
ALTER TABLE dynamic_routes ADD COLUMN file_modified_at TIMESTAMP;
```

#### 1.2 更新数据模型
```rust
// src/db/models.rs
pub enum StorageType {
    Memory,
    File,
}

impl DynamicRoute {
    pub fn storage_type(&self) -> StorageType {
        match self.storage_type.as_str() {
            "file" => StorageType::File,
            _ => StorageType::Memory,
        }
    }
}
```

### 阶段二：存储适配器实现（3-4天）

#### 2.1 创建存储适配器接口
```rust
// src/services/storage_adapters.rs
pub trait RouteStorageAdapter: Send + Sync {
    /// 加载路由内容
    async fn load_content(&self, route: &DynamicRoute) -> Result<RouteContent, StorageError>;

    /// 检查内容是否需要重新加载
    fn needs_reload(&self, route: &DynamicRoute) -> bool;

    /// 获取内容哈希（用于缓存验证）
    fn content_hash(&self, route: &DynamicRoute) -> Option<String>;
}

pub struct RouteContent {
    pub content_type: String,
    pub body: Vec<u8>,
    pub headers: HashMap<String, String>,
}
```

#### 2.2 实现内存存储适配器
```rust
pub struct MemoryStorageAdapter;

impl RouteStorageAdapter for MemoryStorageAdapter {
    async fn load_content(&self, route: &DynamicRoute) -> Result<RouteContent, StorageError> {
        // 从 handler_config 直接读取内容
        let content = route.handler_config.get("content")
            .and_then(|v| v.as_str())
            .ok_or(StorageError::MissingContent)?;

        Ok(RouteContent {
            content_type: "text/html".to_string(),
            body: content.as_bytes().to_vec(),
            headers: HashMap::new(),
        })
    }

    fn needs_reload(&self, _route: &DynamicRoute) -> bool {
        // 内存内容不需要重新加载
        false
    }

    fn content_hash(&self, route: &DynamicRoute) -> Option<String> {
        route.handler_config.get("content")
            .and_then(|v| v.as_str())
            .map(|s| format!("{:x}", md5::compute(s.as_bytes())))
    }
}
```

#### 2.3 实现文件存储适配器
```rust
pub struct FileStorageAdapter {
    base_path: PathBuf,
}

impl FileStorageAdapter {
    pub fn new() -> Self {
        Self {
            base_path: PathBuf::from("./routes"),
        }
    }

    fn resolve_file_path(&self, route: &DynamicRoute) -> Result<PathBuf, StorageError> {
        let file_path = route.file_path.as_ref()
            .ok_or(StorageError::MissingFilePath)?;

        let full_path = self.base_path.join(file_path);

        if !full_path.exists() {
            return Err(StorageError::FileNotFound(full_path));
        }

        Ok(full_path)
    }
}

impl RouteStorageAdapter for FileStorageAdapter {
    async fn load_content(&self, route: &DynamicRoute) -> Result<RouteContent, StorageError> {
        let file_path = self.resolve_file_path(route)?;

        let content = tokio::fs::read(&file_path).await
            .map_err(|e| StorageError::ReadError(e))?;

        let content_type = mime_guess::from_path(&file_path)
            .first_or_octet_stream()
            .to_string();

        Ok(RouteContent {
            content_type,
            body: content,
            headers: HashMap::new(),
        })
    }

    fn needs_reload(&self, route: &DynamicRoute) -> bool {
        if route.storage_type() != StorageType::File {
            return false;
        }

        match self.resolve_file_path(route) {
            Ok(file_path) => {
                match file_path.metadata() {
                    Ok(metadata) => {
                        match metadata.modified() {
                            Ok(modified) => {
                                match route.file_modified_at {
                                    Some(db_modified) => modified > db_modified,
                                    None => true,
                                }
                            }
                            Err(_) => false,
                        }
                    }
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }

    fn content_hash(&self, route: &DynamicRoute) -> Option<String> {
        match self.resolve_file_path(route) {
            Ok(file_path) => {
                match std::fs::read(&file_path) {
                    Ok(content) => Some(format!("{:x}", md5::compute(content))),
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    }
}
```

### 阶段三：路由加载服务重构（2-3天）

#### 3.1 更新 DynamicRouteService
```rust
// src/services/dynamic_route_service.rs
pub struct DynamicRouteService {
    route_table: Arc<RouteTable>,
    repository: DynamicRouteRepository,
    memory_adapter: Arc<MemoryStorageAdapter>,
    file_adapter: Arc<FileStorageAdapter>,
    cache: Arc<RwLock<HashMap<i64, CachedRoute>>>,
}

struct CachedRoute {
    content: RouteContent,
    loaded_at: DateTime<Utc>,
    hash: String,
}

impl DynamicRouteService {
    pub async fn load_route(&self, route_id: i64) -> Result<(), Box<dyn Error>> {
        let route = self.repository.get_by_id(route_id).await?
            .ok_or("Route not found")?;

        let adapter: Arc<dyn RouteStorageAdapter> = match route.storage_type() {
            StorageType::Memory => self.memory_adapter.clone(),
            StorageType::File => self.file_adapter.clone(),
        };

        let content = adapter.load_content(&route).await?;
        let hash = adapter.content_hash(&route).unwrap_or_default();

        // 创建路由条目
        let route_entry = self.create_route_entry(&route, &content)?;

        // 插入路由表
        self.route_table.insert(route.path.clone(), Box::new(route_entry));

        // 更新缓存
        let mut cache = self.cache.write().await;
        cache.insert(route_id, CachedRoute {
            content,
            loaded_at: Utc::now(),
            hash,
        });

        Ok(())
    }

    pub async fn reload_if_needed(&self, route_id: i64) -> Result<bool, Box<dyn Error>> {
        let route = self.repository.get_by_id(route_id).await?
            .ok_or("Route not found")?;

        let adapter: Arc<dyn RouteStorageAdapter> = match route.storage_type() {
            StorageType::Memory => self.memory_adapter.clone(),
            StorageType::File => self.file_adapter.clone(),
        };

        if adapter.needs_reload(&route) {
            self.load_route(route_id).await?;
            return Ok(true);
        }

        Ok(false)
    }
}
```

#### 3.2 创建路由条目工厂
```rust
impl DynamicRouteService {
    fn create_route_entry(
        &self,
        route: &DynamicRoute,
        content: &RouteContent,
    ) -> Result<Box<dyn RouteEntry>, Box<dyn Error>> {
        match route.handler_type {
            HandlerType::Static => {
                Ok(Box::new(SimpleRoute::from_bytes(
                    content.body.clone(),
                    content.content_type.clone(),
                )))
            }
            HandlerType::Template => {
                Ok(Box::new(TemplateRoute::new(
                    content.body.clone(),
                    route.handler_config.clone(),
                )))
            }
            HandlerType::Redirect => {
                let target = route.handler_config.get("target")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing redirect target")?;
                Ok(Box::new(RedirectRoute::new(target)))
            }
            _ => Err("Unsupported handler type".into()),
        }
    }
}
```

### 阶段四：文件管理功能（2-3天）

#### 4.1 创建 routes 目录管理工具
```rust
// src/services/route_file_manager.rs
pub struct RouteFileManager {
    base_path: PathBuf,
}

impl RouteFileManager {
    pub fn new() -> Self {
        let base_path = PathBuf::from("./routes");
        if !base_path.exists() {
            std::fs::create_dir_all(&base_path)
                .expect("Failed to create routes directory");
        }
        Self { base_path }
    }

    /// 创建新的路由文件
    pub async fn create_route_file(
        &self,
        path: &str,
        content: &str,
    ) -> Result<String, FileManagerError> {
        let file_path = self.base_path.join(path);

        // 确保父目录存在
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // 写入文件
        tokio::fs::write(&file_path, content).await?;

        Ok(file_path.to_string_lossy().to_string())
    }

    /// 更新路由文件
    pub async fn update_route_file(
        &self,
        path: &str,
        content: &str,
    ) -> Result<(), FileManagerError> {
        let file_path = self.base_path.join(path);
        tokio::fs::write(&file_path, content).await?;
        Ok(())
    }

    /// 删除路由文件
    pub async fn delete_route_file(&self, path: &str) -> Result<(), FileManagerError> {
        let file_path = self.base_path.join(path);
        tokio::fs::remove_file(&file_path).await?;
        Ok(())
    }

    /// 读取路由文件
    pub async fn read_route_file(&self, path: &str) -> Result<String, FileManagerError> {
        let file_path = self.base_path.join(path);
        let content = tokio::fs::read_to_string(&file_path).await?;
        Ok(content)
    }

    /// 列出所有路由文件
    pub async fn list_route_files(&self) -> Result<Vec<String>, FileManagerError> {
        let mut files = Vec::new();

        let entries = tokio::fs::read_dir(&self.base_path).await?;
        let mut read_dir = entries;

        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                if let Some(relative) = path.strip_prefix(&self.base_path).ok() {
                    files.push(relative.to_string_lossy().to_string());
                }
            }
        }

        Ok(files)
    }
}
```

#### 4.2 集成到路由创建流程
```rust
// src/handlers/api_handlers/dynamic_routes/create.rs
pub async fn create_route(
    req: actix_web::HttpRequest,
    route_data: web::Json<CreateRouteRequest>,
    state: web::Data<AppState>,
) -> HttpResponse {
    // ... 现有验证逻辑 ...

    // 如果是文件类型，创建对应的文件
    if route_data.storage_type == Some("file".to_string()) {
        let content = route_data.handler_config.get("content")
            .and_then(|v| v.as_str())
            .ok_or("Missing content for file storage");

        if let Ok(content) = content {
            let file_path = format!("{}.html", route_data.path.trim_start_matches('/').replace('/', "_"));

            match state.route_file_manager().create_route_file(&file_path, content).await {
                Ok(created_path) => {
                    // 记录文件路径
                    // ... 保存到数据库 ...
                }
                Err(e) => {
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "success": false,
                        "message": format!("Failed to create route file: {}", e)
                    }));
                }
            }
        }
    }

    // ... 继续创建路由 ...
}
```

### 阶段五：热重载机制（1-2天）

#### 5.1 实现文件监听器
```rust
// src/services/file_watcher.rs
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

pub struct RouteFileWatcher {
    _watcher: RecommendedWatcher,
}

impl RouteFileWatcher {
    pub async fn new(service: Arc<DynamicRouteService>) -> Result<Self, Box<dyn Error>> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        let mut watcher = notify::recommended_watcher(move |res| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        })?;

        watcher.watch(Path::new("./routes"), RecursiveMode::Recursive)?;

        // 启动处理任务
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                if let Err(e) = Self::handle_file_change(event, &service).await {
                    tracing::error!("Failed to handle file change: {}", e);
                }
            }
        });

        Ok(Self { _watcher: watcher })
    }

    async fn handle_file_change(
        event: notify::Event,
        service: &Arc<DynamicRouteService>,
    ) -> Result<(), Box<dyn Error>> {
        for path in event.paths {
            if path.extension().and_then(|s| s.to_str()) == Some("html") {
                // 查找对应的路由
                if let Some(route_id) = service.find_route_by_file_path(&path).await {
                    tracing::info!("Reloading route {} due to file change", route_id);
                    service.load_route(route_id).await?;
                }
            }
        }
        Ok(())
    }
}
```

#### 5.2 在主程序中启动文件监听
```rust
// src/main.rs
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ... 现有初始化代码 ...

    // 启动文件监听器
    if let Err(e) = RouteFileWatcher::new(state.dynamic_route_service().clone()).await {
        tracing::warn!("Failed to start file watcher: {}", e);
    }

    // ... 启动HTTP服务器 ...
}
```

### 阶段六：管理界面增强（2-3天）

#### 6.1 添加存储类型选择
在前端管理界面添加存储类型选择器：
- Memory: 直接编辑内容
- File: 上传或编辑文件

#### 6.2 添加文件管理功能
- 文件列表查看
- 文件内容编辑
- 文件上传/下载
- 文件删除

#### 6.3 添加热重载状态显示
- 显示最后加载时间
- 显示文件修改状态
- 手动重新加载按钮

### 阶段七：测试与优化（2-3天）

#### 7.1 单元测试
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_storage_adapter() {
        let adapter = MemoryStorageAdapter;
        let route = create_test_route(StorageType::Memory);

        let content = adapter.load_content(&route).await.unwrap();
        assert!(!content.body.is_empty());
    }

    #[tokio::test]
    async fn test_file_storage_adapter() {
        let adapter = FileStorageAdapter::new();
        let route = create_test_route_with_file();

        let content = adapter.load_content(&route).await.unwrap();
        assert!(!content.body.is_empty());
    }

    #[tokio::test]
    async fn test_hot_reload() {
        // 测试文件修改后自动重新加载
    }
}
```

#### 7.2 性能优化
- 实现内容缓存
- 添加并发加载支持
- 优化文件读取性能

#### 7.3 错误处理增强
- 文件不存在时的友好错误提示
- 文件权限错误处理
- 磁盘空间不足处理

## 迁移策略

### 现有数据迁移
```sql
-- 将现有路由标记为内存类型
UPDATE dynamic_routes SET storage_type = 'memory' WHERE storage_type IS NULL;

-- 对于需要迁移到文件存储的路由
-- 1. 提取内容到文件
-- 2. 更新 file_path 字段
-- 3. 更新 storage_type 为 'file'
```

### 向后兼容
- 保持现有的 `handler_config` 字段兼容性
- 如果 `storage_type` 未指定，默认使用内存存储
- 文件路径为空时，自动回退到内存存储

## 时间线

| 阶段 | 任务 | 预计时间 |
|------|------|----------|
| 1 | 数据库架构扩展 | 1-2天 |
| 2 | 存储适配器实现 | 3-4天 |
| 3 | 路由加载服务重构 | 2-3天 |
| 4 | 文件管理功能 | 2-3天 |
| 5 | 热重载机制 | 1-2天 |
| 6 | 管理界面增强 | 2-3天 |
| 7 | 测试与优化 | 2-3天 |
| **总计** | | **13-20天** |

## 风险与挑战

### 技术风险
1. **文件系统权限**：确保程序有读写 `./routes` 目录的权限
2. **并发访问**：多个请求同时修改同一文件时的锁机制
3. **性能影响**：频繁的文件监听可能影响性能

### 解决方案
1. 在启动时检查并创建必要的目录
2. 使用文件锁和事务处理并发修改
3. 实现事件去重和批量处理

## 依赖项

需要添加的 Cargo 依赖：
```toml
[dependencies]
# 文件监听
notify = "6.0"

# MIME 类型检测
mime_guess = "2.0"

# 文件哈希计算
md5 = "0.7"

# 文件系统操作（已有）
tokio = { version = "1.0", features = ["full"] }
```

## 总结

本计划提供了一个完整的动态路由存储实现方案，包括：

1. **灵活的存储类型**：支持内存和文件两种存储方式
2. **高性能加载**：通过适配器模式和缓存优化性能
3. **热重载支持**：文件修改后自动生效
4. **完善的管理工具**：提供文件管理功能
5. **向后兼容**：保持现有API的兼容性

按照此计划实施，将大大提升动态路由系统的灵活性和可维护性。