warning: unused imports: `PassageRepository` and `Repository`
  --> src/handlers/page_handlers.rs:11:31
   |
11 | use crate::db::repositories::{PassageRepository, Repository};
   |                               ^^^^^^^^^^^^^^^^^  ^^^^^^^^^^
   |
   = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `std::sync::Arc`
  --> src/handlers/page_handlers.rs:12:5
   |
12 | use std::sync::Arc;
   |     ^^^^^^^^^^^^^^

warning: unused import: `Duration`
 --> src/middleware/logging.rs:6:17
  |
6 | use std::time::{Duration, Instant};
  |                 ^^^^^^^^

warning: unreachable expression
   --> src/handlers/api_handlers/attachments.rs:293:5
    |
269 | /     match attachment_repo.create(&attachment).await {
270 | |         Ok(_) => {
271 | |             return HttpResponse::Ok().json(UploadResponse {
272 | |                 success: true,
...   |
291 | |     }
    | |_____- any code following this `match` expression is unreachable, as all arms diverge
292 |       
293 | /     HttpResponse::BadRequest().json(UploadResponse {
294 | |         success: false,
295 | |         message: "没有上传文件".to_string(),
296 | |         data: None,
297 | |     })
    | |______^ unreachable expression
    |
    = note: `#[warn(unreachable_code)]` (part of `#[warn(unused)]`) on by default

warning: unused import: `p256::elliptic_curve::group::GroupEncoding`
  --> src/handlers/api_handlers/crypto.rs:86:13
   |
86 |         use p256::elliptic_curve::group::GroupEncoding;
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: unused variable: `req`
    --> src/handlers/api_handlers/passage.rs:1240:5
     |
1240 |     req: HttpRequest,
     |     ^^^ help: if this is intentional, prefix it with an underscore: `_req`
     |
     = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: variable `file_field_name` is assigned to, but never used
   --> src/handlers/api_handlers/attachments.rs:155:9
    |
155 |     let mut file_field_name: Option<String> = None;
    |         ^^^^^^^^^^^^^^^^^^^
    |
    = note: consider using `_file_field_name` instead

warning: value assigned to `file_field_name` is never read
   --> src/handlers/api_handlers/attachments.rs:197:13
    |
197 |             file_field_name = Some(filename.clone());
    |             ^^^^^^^^^^^^^^^
    |
    = help: maybe it is overwritten before being read?
    = note: `#[warn(unused_assignments)]` (part of `#[warn(unused)]`) on by default

warning: value assigned to `passage_id_str` is never read
   --> src/handlers/api_handlers/attachments.rs:172:38
    |
172 |             let mut passage_id_str = String::new();
    |                                      ^^^^^^^^^^^^^
    |
    = help: maybe it is overwritten before being read?

warning: unused variable: `passage_repo`
   --> src/handlers/api_handlers/sync.rs:274:5
    |
274 |     passage_repo: &PassageRepository,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_passage_repo`

warning: unused variable: `passage_repo`
   --> src/handlers/api_handlers/sync.rs:303:5
    |
303 |     passage_repo: &PassageRepository,
    |     ^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_passage_repo`

warning: constant `COMPRESSED_CONTENT_TYPES` is never used
  --> src/main.rs:22:7
   |
22 | const COMPRESSED_CONTENT_TYPES: [&str; 6] = [
   |       ^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: function `is_already_compressed` is never used
  --> src/main.rs:32:4
   |
32 | fn is_already_compressed(content_type: &str) -> bool {
   |    ^^^^^^^^^^^^^^^^^^^^^

warning: function `optimized_compress` is never used
  --> src/main.rs:37:4
   |
37 | fn optimized_compress() -> actix_middleware::Condition<actix_middleware::Comp...
   |    ^^^^^^^^^^^^^^^^^^

warning: function `create_passage_context_with_article` is never used
  --> src/handlers/page_handlers.rs:29:4
   |
29 | fn create_passage_context_with_article(passage: &Passage) -> TeraContext {
   |    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: function `remove_first_h1` is never used
  --> src/handlers/page_handlers.rs:49:4
   |
49 | fn remove_first_h1(content: &str) -> String {
   |    ^^^^^^^^^^^^^^^

warning: field `algorithm` is never read
  --> src/handlers/api_handlers/auth.rs:17:9
   |
 6 | pub struct LoginRequest {
   |            ------------ field in this struct
...
17 |     pub algorithm: String,
   |         ^^^^^^^^^
   |
   = note: `LoginRequest` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis

warning: field `algorithm` is never read
  --> src/handlers/api_handlers/auth.rs:35:9
   |
21 | pub struct RegisterRequest {
   |            --------------- field in this struct
...
35 |     pub algorithm: String,
   |         ^^^^^^^^^
   |
   = note: `RegisterRequest` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis

warning: field `original_content` is never read
  --> src/handlers/api_handlers/passage.rs:36:9
   |
33 | pub struct CreatePassageRequest {
   |            -------------------- field in this struct
...
36 |     pub original_content: Option<String>,
   |         ^^^^^^^^^^^^^^^^
   |
   = note: `CreatePassageRequest` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis

warning: function `admin_list` is never used
   --> src/handlers/api_handlers/passage.rs:138:14
    |
138 | pub async fn admin_list(
    |              ^^^^^^^^^^

warning: function `get_by_id` is never used
   --> src/handlers/api_handlers/passage.rs:356:14
    |
356 | pub async fn get_by_id(
    |              ^^^^^^^^^

warning: struct `AboutContent` is never constructed
 --> src/handlers/api_handlers/about.rs:8:12
  |
8 | pub struct AboutContent {
  |            ^^^^^^^^^^^^

warning: struct `PaginatedResponse` is never constructed
  --> src/handlers/api_handlers/comment.rs:34:12
   |
34 | pub struct PaginatedResponse<T> {
   |            ^^^^^^^^^^^^^^^^^

warning: struct `Pagination` is never constructed
  --> src/handlers/api_handlers/comment.rs:41:12
   |
41 | pub struct Pagination {
   |            ^^^^^^^^^^

warning: field `session_id` is never read
  --> src/handlers/api_handlers/crypto.rs:32:9
   |
31 | pub struct ECCSession {
   |            ---------- field in this struct
32 |     pub session_id: String,
   |         ^^^^^^^^^^
   |
   = note: `ECCSession` has a derived impl for the trait `Clone`, but this is intentionally ignored during dead code analysis

warning: fields `synced_count`, `updated_count`, and `deleted_count` are never read
  --> src/handlers/api_handlers/sync.rs:19:9
   |
18 | pub struct SyncResult {
   |            ---------- fields in this struct
19 |     pub synced_count: usize,
   |         ^^^^^^^^^^^^
20 |     pub updated_count: usize,
   |         ^^^^^^^^^^^^^
21 |     pub deleted_count: usize,
   |         ^^^^^^^^^^^^^
   |
   = note: `SyncResult` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis

warning: struct `DbStatsResponse` is never constructed
 --> src/handlers/api_handlers/db_stats.rs:7:12
  |
7 | pub struct DbStatsResponse {
  |            ^^^^^^^^^^^^^^^

warning: struct `HealthCheckResponse` is never constructed
  --> src/handlers/api_handlers/db_stats.rs:32:12
   |
32 | pub struct HealthCheckResponse {
   |            ^^^^^^^^^^^^^^^^^^^

warning: struct `HealthCheckData` is never constructed
  --> src/handlers/api_handlers/db_stats.rs:38:12
   |
38 | pub struct HealthCheckData {
   |            ^^^^^^^^^^^^^^^

warning: struct `MarkdownPreviewRequest` is never constructed
 --> src/handlers/api_handlers/markdown_preview.rs:8:12
  |
8 | pub struct MarkdownPreviewRequest {
  |            ^^^^^^^^^^^^^^^^^^^^^^

warning: struct `StaticFileService` is never constructed
 --> src/static/mod.rs:7:12
  |
7 | pub struct StaticFileService;
  |            ^^^^^^^^^^^^^^^^^

warning: associated functions `get_etag`, `check_cache`, and `add_cache_headers` are never used
  --> src/static/mod.rs:11:12
   |
 9 | impl StaticFileService {
   | ---------------------- associated functions in this implementation
10 |     /// 获取文件的 ETag
11 |     pub fn get_etag(file_path: &Path) -> Option<String> {
   |            ^^^^^^^^
...
28 |     pub fn check_cache(req: &HttpRequest, file_path: &Path) -> Option<HttpRes...
   |            ^^^^^^^^^^^
...
45 |     pub fn add_cache_headers(mut response: HttpResponse, max_age: u32) -> Htt...
   |            ^^^^^^^^^^^^^^^^^

warning: function `get_cache_max_age` is never used
  --> src/static/mod.rs:59:8
   |
59 | pub fn get_cache_max_age(file_path: &Path) -> u32 {
   |        ^^^^^^^^^^^^^^^^^

warning: struct `Visitor` is never constructed
  --> src/db/models.rs:41:12
   |
41 | pub struct Visitor {
   |            ^^^^^^^

warning: struct `ArticleView` is never constructed
  --> src/db/models.rs:51:12
   |
51 | pub struct ArticleView {
   |            ^^^^^^^^^^^

warning: methods `get_published_with_count`, `delete`, and `get_uuids_by_ids` are never used
   --> src/db/repositories.rs:256:18
    |
 51 | impl PassageRepository {
    | ---------------------- methods in this implementation
...
256 |     pub async fn get_published_with_count(&self, limit: i64, offset: i64) ->...
    |                  ^^^^^^^^^^^^^^^^^^^^^^^^
...
322 |     pub async fn delete(&self, id: i64) -> Result<(), Box<dyn std::error::Er...
    |                  ^^^^^^
...
349 |     pub async fn get_uuids_by_ids(&self, ids: Vec<i64>) -> Result<Vec<String...
    |                  ^^^^^^^^^^^^^^^^

warning: method `get_by_id` is never used
   --> src/db/repositories.rs:413:18
    |
392 | impl CommentRepository {
    | ---------------------- method in this implementation
...
413 |     pub async fn get_by_id(&self, id: i64) -> Result<Comment, Box<dyn std::e...
    |                  ^^^^^^^^^

warning: method `record_view` is never used
   --> src/db/repositories.rs:518:18
    |
512 | impl ArticleViewRepository {
    | -------------------------- method in this implementation
...
518 |     pub async fn record_view(&self, passage_uuid: &str, ip: &str, user_agent...
    |                  ^^^^^^^^^^^

warning: associated functions `get_all` and `get_by_category` are never used
   --> src/db/repositories.rs:784:12
    |
739 | impl SettingRepository {
    | ---------------------- associated functions in this implementation
...
784 |     pub fn get_all(conn: &rusqlite::Connection) -> Result<Vec<Setting>, Box<...
    |            ^^^^^^^
...
807 |     pub fn get_by_category(conn: &rusqlite::Connection, category: &str) -> R...
    |            ^^^^^^^^^^^^^^^

warning: method `get_all_without_pagination` is never used
   --> src/db/repositories.rs:908:18
    |
835 | impl CategoryRepository {
    | ----------------------- method in this implementation
...
908 |     pub async fn get_all_without_pagination(&self) -> Result<Vec<Category>, ...
    |                  ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: method `get_all_without_pagination` is never used
    --> src/db/repositories.rs:1085:18
     |
 984 | impl TagRepository {
     | ------------------ method in this implementation
...
1085 |     pub async fn get_all_without_pagination(&self) -> Result<Vec<Tag>, Box<...
     |                  ^^^^^^^^^^^^^^^^^^^^^^^^^^

warning: method `count` is never used
    --> src/db/repositories.rs:1472:18
     |
1420 | impl AttachmentRepository {
     | ------------------------- method in this implementation
...
1472 |     pub async fn count(&self) -> Result<i64, Box<dyn std::error::Error>> {
     |                  ^^^^^

warning: fields `album`, `year`, `genre`, and `duration` are never read
  --> src/audio_metadata.rs:8:9
   |
 5 | pub struct AudioMetadata {
   |            ------------- fields in this struct
...
 8 |     pub album: Option<String>,
   |         ^^^^^
 9 |     pub year: Option<u32>,
   |         ^^^^
10 |     pub genre: Option<String>,
   |         ^^^^^
11 |     pub duration: Option<f64>, // 秒
   |         ^^^^^^^^
   |
   = note: `AudioMetadata` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: function `format_duration` is never used
   --> src/audio_metadata.rs:156:8
    |
156 | pub fn format_duration(seconds: f64) -> String {
    |        ^^^^^^^^^^^^^^^

warning: fields `synced_count`, `updated_count`, and `deleted_count` are never read
   --> src/music_sync.rs:317:9
    |
316 | pub struct SyncResult {
    |            ---------- fields in this struct
317 |     pub synced_count: usize,
    |         ^^^^^^^^^^^^
318 |     pub updated_count: usize,
    |         ^^^^^^^^^^^^^
319 |     pub deleted_count: usize,
    |         ^^^^^^^^^^^^^
    |
    = note: `SyncResult` has a derived impl for the trait `Debug`, but this is intentionally ignored during dead code analysis

warning: function `has_embedded_file` is never used
   --> src/embedded.rs:105:8
    |
105 | pub fn has_embedded_file(path: &str) -> bool {
    |        ^^^^^^^^^^^^^^^^^

warning: variants `Setting`, `SettingCategory`, `Passage`, `PassageList`, and `Stats` are never constructed
  --> src/cache/mod.rs:16:5
   |
14 | pub enum CacheKey {
   |          -------- variants in this enum
15 |     /// 设置缓存键
16 |     Setting(String),
   |     ^^^^^^^
17 |     /// 设置分类缓存键
18 |     SettingCategory(String),
   |     ^^^^^^^^^^^^^^^
19 |     /// 文章缓存键
20 |     Passage(String),
   |     ^^^^^^^
21 |     /// 文章列表缓存键
22 |     PassageList { limit: i64, page: i64 },
   |     ^^^^^^^^^^^
23 |     /// 统计数据缓存键
24 |     Stats(String),
   |     ^^^^^
   |
   = note: `CacheKey` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: enum `CacheValue` is never used
  --> src/cache/mod.rs:29:10
   |
29 | pub enum CacheValue {
   |          ^^^^^^^^^^

warning: fields `setting_ttl`, `passage_ttl`, and `stats_ttl` are never read
  --> src/cache/mod.rs:42:9
   |
38 | pub struct CacheConfig {
   |            ----------- fields in this struct
...
42 |     pub setting_ttl: u64,
   |         ^^^^^^^^^^^
43 |     /// 文章缓存 TTL (秒)
44 |     pub passage_ttl: u64,
   |         ^^^^^^^^^^^
45 |     /// 统计数据缓存 TTL (秒)
46 |     pub stats_ttl: u64,
   |         ^^^^^^^^^
   |
   = note: `CacheConfig` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: fields `value` and `expires_at` are never read
  --> src/cache/mod.rs:63:5
   |
62 | struct CacheEntry {
   |        ---------- fields in this struct
63 |     value: String,
   |     ^^^^^
64 |     expires_at: chrono::DateTime<chrono::Utc>,
   |     ^^^^^^^^^^
   |
   = note: `CacheEntry` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: associated items `new` and `is_expired` are never used
  --> src/cache/mod.rs:68:8
   |
67 | impl CacheEntry {
   | --------------- associated items in this implementation
68 |     fn new(value: String, ttl_seconds: u64) -> Self {
   |        ^^^
...
73 |     fn is_expired(&self) -> bool {
   |        ^^^^^^^^^^

warning: fields `config` and `cache` are never read
  --> src/cache/mod.rs:80:5
   |
79 | pub struct AppCache {
   |            -------- fields in this struct
80 |     config: CacheConfig,
   |     ^^^^^^
81 |     cache: Arc<RwLock<LruCache<CacheKey, CacheEntry>>>,
   |     ^^^^^

warning: multiple methods are never used
   --> src/cache/mod.rs:95:18
    |
 84 | impl AppCache {
    | ------------- methods in this implementation
...
 95 |     pub async fn get(&self, key: &CacheKey) -> Option<String> {
    |                  ^^^
...
110 |     pub async fn set(&self, key: CacheKey, value: String) {
    |                  ^^^
...
117 |     pub async fn invalidate(&self, key: &CacheKey) {
    |                  ^^^^^^^^^^
...
123 |     pub async fn invalidate_category(&self, category: &str) {
    |                  ^^^^^^^^^^^^^^^^^^^
...
142 |     pub async fn invalidate_settings(&self) {
    |                  ^^^^^^^^^^^^^^^^^^^
...
156 |     pub async fn invalidate_passages(&self) {
    |                  ^^^^^^^^^^^^^^^^^^^
...
170 |     pub async fn invalidate_stats(&self) {
    |                  ^^^^^^^^^^^^^^^^
...
184 |     pub async fn clear(&self) {
    |                  ^^^^^
...
190 |     pub async fn stats(&self) -> CacheStats {
    |                  ^^^^^
...
199 |     fn get_ttl_for_key(&self, key: &CacheKey) -> u64 {
    |        ^^^^^^^^^^^^^^^

warning: struct `CacheStats` is never constructed
   --> src/cache/mod.rs:210:12
    |
210 | pub struct CacheStats {
    |            ^^^^^^^^^^

warning: struct `SettingCache` is never constructed
   --> src/cache/mod.rs:218:12
    |
218 | pub struct SettingCache {
    |            ^^^^^^^^^^^^

warning: associated items `new`, `get`, `set`, `invalidate`, and `invalidate_all` are never used
   --> src/cache/mod.rs:223:12
    |
222 | impl SettingCache {
    | ----------------- associated items in this implementation
223 |     pub fn new(cache: Arc<AppCache>) -> Self {
    |            ^^^
...
228 |     pub async fn get(&self, key: &str) -> Option<String> {
    |                  ^^^
...
233 |     pub async fn set(&self, key: String, value: String) {
    |                  ^^^
...
238 |     pub async fn invalidate(&self, key: &str) {
    |                  ^^^^^^^^^^
...
243 |     pub async fn invalidate_all(&self) {
    |                  ^^^^^^^^^^^^^^

warning: struct `StatsCache` is never constructed
   --> src/cache/mod.rs:249:12
    |
249 | pub struct StatsCache {
    |            ^^^^^^^^^^

warning: associated items `new`, `get`, `set`, `invalidate`, and `invalidate_all` are never used
   --> src/cache/mod.rs:254:12
    |
253 | impl StatsCache {
    | --------------- associated items in this implementation
254 |     pub fn new(cache: Arc<AppCache>) -> Self {
    |            ^^^
...
259 |     pub async fn get(&self, key: &str) -> Option<String> {
    |                  ^^^
...
264 |     pub async fn set(&self, key: String, value: String) {
    |                  ^^^
...
269 |     pub async fn invalidate(&self, key: &str) {
    |                  ^^^^^^^^^^
...
274 |     pub async fn invalidate_all(&self) {
    |                  ^^^^^^^^^^^^^^

warning: field `channel_buffer` is never read
  --> src/view_batch.rs:32:9
   |
26 | pub struct BatchConfig {
   |            ----------- field in this struct
...
32 |     pub channel_buffer: usize,
   |         ^^^^^^^^^^^^^^
   |
   = note: `BatchConfig` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis

warning: method `pending_count` is never used
   --> src/view_batch.rs:164:12
    |
 51 | impl ViewBatchProcessor {
    | ----------------------- method in this implementation
...
164 |     pub fn pending_count(&self) -> usize {
    |            ^^^^^^^^^^^^^

warning: `rustblog` (bin "rustblog") generated 60 warnings (run `cargo fix --bin "rustblog" -p rustblog` to apply 6 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.25s
     Running `target/debug/rustblog`
🚀 启动 RustBlog 服务器...
📡 访问地址: http://127.0.0.1:8080
📁 模板目录: /home/swordreforge/project/rustblog/templates
📁 静态文件目录: /home/swordreforge/project/rustblog/static
📁 数据库路径: /home/swordreforge/project/rustblog/./data/blog.db
📁 GeoIP 数据库: /home/swordreforge/project/rustblog/./data/GeoLite2-City.mmdb
💾 模板缓存: 启用
🔒 TLS: 禁用
📊 日志级别: info
📦 资源初始化...
📦 释放嵌入的资源...
  ✓ 创建目录: attachments
  ✓ 创建目录: data
  ✓ 创建目录: markdown
