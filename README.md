# simple-blog
## 也许是最小构建包体的动态博客,但功能全面
### 一.包体极小(<10M)
```shell
.rwxr-xr-x 9.4M nobody  5 2月  00:34 rustblog
```
### 二.运行内存极低,100多m运存即可运行，峰值运行不超过100M，512m服务器即可正常运行
```shell
->lsof -i :8080
COMMAND      PID         USER   FD   TYPE DEVICE SIZE/OFF        NODE NAME
rustblog   2511793      nobody  17u  IPv4 609744   0t0    TCP *:http-alt (LISTEN)
-> ps -p 2511793  -o pid,rss,vsize,pcpu,pmem,cmd
  PID    RSS    VSZ  %CPU %MEM     CMD
2511793 33380 149608  215  0.1 ./rustblog
```
### 三.无畏并发
```shell
wrk -t2 -c100 -d30s http://localhost:8080/api/passages
Running 30s test @ http://localhost:8080/api/passages
  2 threads and 100 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     5.29ms    1.16ms  23.50ms   71.10%
    Req/Sec     9.50k   675.01    11.09k    64.17%
  567306 requests in 30.01s, 134.17MB read
Requests/sec:  18901.25
Transfer/sec:      4.47MB
```
### 四.单文件部署,零插件依赖
#### 1.集成**常用**功能(文章,评论,后台,附件)，无须也不必要引入任何插件系统
#### 2.sqlite文件类型数据库,无须任何其他后台服务\n
#### 3.嵌入式模板(rust-embed)只释放可变的文件目录\n
### 六.致谢
感谢以下下开源库及其它们对应的底层库的依赖的作者的贡献
```rust
cargo tree --prefix none --depth 1 | tail -n +2
actix-files v0.6.9
actix-multipart v0.7.2
actix-web v4.12.1
aes-gcm v0.10.3
argon2 v0.5.3
async-trait v0.1.89 (proc-macro)
base64 v0.22.1
chrono v0.4.43
clap v4.5.56
ecdsa v0.16.9
elliptic-curve v0.13.8
futures-util v0.3.31
hex v0.4.3
id3 v0.6.6
jsonwebtoken v10.3.0
lazy_static v1.5.0
maxminddb v0.27.1
md-5 v0.10.6
memmap2 v0.9.9
metaflac v0.2.8
mime_guess v2.0.5
once_cell v1.21.3
p256 v0.13.2
pulldown-cmark v0.12.2
r2d2 v0.8.10
r2d2_sqlite v0.25.0
rand v0.8.5
regex v1.12.2
rusqlite v0.32.1
rust-embed v8.11.0
serde v1.0.228
serde_json v1.0.149
sha2 v0.10.9
snowflake-id-generator v0.4.0
spki v0.7.3
tera v1.20.1
tokio v1.49.0
toml v0.8.23
urlencoding v2.1.3
```
