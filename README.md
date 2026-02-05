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
感谢以下下开源库的作者的贡献
```rust
 cargo tree
rustblog v1.1.4 (/home/nobody/project/rustblog)
├── actix-files v0.6.9
│   ├── actix-http v3.11.2
│   │   ├── actix-codec v0.5.2
│   │   │   ├── bitflags v2.10.0
│   │   │   ├── bytes v1.11.0
│   │   │   ├── futures-core v0.3.31
│   │   │   ├── futures-sink v0.3.31
│   │   │   ├── memchr v2.7.6
│   │   │   ├── pin-project-lite v0.2.16
│   │   │   ├── tokio v1.49.0
│   │   │   │   ├── bytes v1.11.0
│   │   │   │   ├── libc v0.2.180
│   │   │   │   ├── mio v1.1.1
│   │   │   │   │   ├── libc v0.2.180
│   │   │   │   │   └── log v0.4.29
│   │   │   │   ├── parking_lot v0.12.5
│   │   │   │   │   ├── lock_api v0.4.14
│   │   │   │   │   │   └── scopeguard v1.2.0
│   │   │   │   │   └── parking_lot_core v0.9.12
│   │   │   │   │       ├── cfg-if v1.0.4
│   │   │   │   │       ├── libc v0.2.180
│   │   │   │   │       └── smallvec v1.15.1
│   │   │   │   ├── pin-project-lite v0.2.16
│   │   │   │   ├── signal-hook-registry v1.4.8
│   │   │   │   │   ├── errno v0.3.14
│   │   │   │   │   │   └── libc v0.2.180
│   │   │   │   │   └── libc v0.2.180
│   │   │   │   ├── socket2 v0.6.2
│   │   │   │   │   └── libc v0.2.180
│   │   │   │   └── tokio-macros v2.6.0 (proc-macro)
│   │   │   │       ├── proc-macro2 v1.0.106
│   │   │   │       │   └── unicode-ident v1.0.22
│   │   │   │       ├── quote v1.0.44
│   │   │   │       │   └── proc-macro2 v1.0.106 (*)
│   │   │   │       └── syn v2.0.114
│   │   │   │           ├── proc-macro2 v1.0.106 (*)
│   │   │   │           ├── quote v1.0.44 (*)
│   │   │   │           └── unicode-ident v1.0.22
│   │   │   ├── tokio-util v0.7.18
│   │   │   │   ├── bytes v1.11.0
│   │   │   │   ├── futures-core v0.3.31
│   │   │   │   ├── futures-sink v0.3.31
│   │   │   │   ├── pin-project-lite v0.2.16
│   │   │   │   └── tokio v1.49.0 (*)
│   │   │   └── tracing v0.1.44
│   │   │       ├── log v0.4.29
│   │   │       ├── pin-project-lite v0.2.16
│   │   │       ├── tracing-attributes v0.1.31 (proc-macro)
│   │   │       │   ├── proc-macro2 v1.0.106 (*)
│   │   │       │   ├── quote v1.0.44 (*)
│   │   │       │   └── syn v2.0.114 (*)
│   │   │       └── tracing-core v0.1.36
│   │   │           └── once_cell v1.21.3
│   │   ├── actix-rt v2.11.0
│   │   │   ├── futures-core v0.3.31
│   │   │   └── tokio v1.49.0 (*)
│   │   ├── actix-service v2.0.3
│   │   │   ├── futures-core v0.3.31
│   │   │   └── pin-project-lite v0.2.16
│   │   ├── actix-utils v3.0.1
│   │   │   ├── local-waker v0.1.4
│   │   │   └── pin-project-lite v0.2.16
│   │   ├── bitflags v2.10.0
│   │   ├── brotli v8.0.2
│   │   │   ├── alloc-no-stdlib v2.0.4
│   │   │   ├── alloc-stdlib v0.2.2
│   │   │   │   └── alloc-no-stdlib v2.0.4
│   │   │   └── brotli-decompressor v5.0.0
│   │   │       ├── alloc-no-stdlib v2.0.4
│   │   │       └── alloc-stdlib v0.2.2 (*)
│   │   ├── bytes v1.11.0
│   │   ├── bytestring v1.5.0
│   │   │   └── bytes v1.11.0
│   │   ├── derive_more v2.1.1
│   │   │   └── derive_more-impl v2.1.1 (proc-macro)
│   │   │       ├── convert_case v0.10.0
│   │   │       │   └── unicode-segmentation v1.12.0
│   │   │       ├── proc-macro2 v1.0.106 (*)
│   │   │       ├── quote v1.0.44 (*)
│   │   │       ├── syn v2.0.114 (*)
│   │   │       └── unicode-xid v0.2.6
│   │   │       [build-dependencies]
│   │   │       └── rustc_version v0.4.1
│   │   │           └── semver v1.0.27
│   │   ├── encoding_rs v0.8.35
│   │   │   └── cfg-if v1.0.4
│   │   ├── foldhash v0.1.5
│   │   ├── futures-core v0.3.31
│   │   ├── http v0.2.12
│   │   │   ├── bytes v1.11.0
│   │   │   ├── fnv v1.0.7
│   │   │   └── itoa v1.0.17
│   │   ├── httparse v1.10.1
│   │   ├── httpdate v1.0.3
│   │   ├── itoa v1.0.17
│   │   ├── language-tags v0.3.2
│   │   ├── mime v0.3.17
│   │   ├── percent-encoding v2.3.2
│   │   ├── pin-project-lite v0.2.16
│   │   ├── smallvec v1.15.1
│   │   ├── tokio v1.49.0 (*)
│   │   ├── tokio-util v0.7.18 (*)
│   │   └── tracing v0.1.44 (*)
│   ├── actix-service v2.0.3 (*)
│   ├── actix-utils v3.0.1 (*)
│   ├── actix-web v4.12.1
│   │   ├── actix-codec v0.5.2 (*)
│   │   ├── actix-http v3.11.2 (*)
│   │   ├── actix-macros v0.2.4 (proc-macro)
│   │   │   ├── quote v1.0.44 (*)
│   │   │   └── syn v2.0.114 (*)
│   │   ├── actix-router v0.5.3
│   │   │   ├── bytestring v1.5.0 (*)
│   │   │   ├── cfg-if v1.0.4
│   │   │   ├── http v0.2.12 (*)
│   │   │   ├── regex-lite v0.1.8
│   │   │   ├── serde v1.0.228
│   │   │   │   ├── serde_core v1.0.228
│   │   │   │   └── serde_derive v1.0.228 (proc-macro)
│   │   │   │       ├── proc-macro2 v1.0.106 (*)
│   │   │   │       ├── quote v1.0.44 (*)
│   │   │   │       └── syn v2.0.114 (*)
│   │   │   └── tracing v0.1.44 (*)
│   │   ├── actix-rt v2.11.0 (*)
│   │   ├── actix-server v2.6.0
│   │   │   ├── actix-rt v2.11.0 (*)
│   │   │   ├── actix-service v2.0.3 (*)
│   │   │   ├── actix-utils v3.0.1 (*)
│   │   │   ├── futures-core v0.3.31
│   │   │   ├── futures-util v0.3.31
│   │   │   │   ├── futures-core v0.3.31
│   │   │   │   ├── futures-macro v0.3.31 (proc-macro)
│   │   │   │   │   ├── proc-macro2 v1.0.106 (*)
│   │   │   │   │   ├── quote v1.0.44 (*)
│   │   │   │   │   └── syn v2.0.114 (*)
│   │   │   │   ├── futures-task v0.3.31
│   │   │   │   ├── pin-project-lite v0.2.16
│   │   │   │   └── pin-utils v0.1.0
│   │   │   ├── mio v1.1.1 (*)
│   │   │   ├── socket2 v0.5.10
│   │   │   │   └── libc v0.2.180
│   │   │   ├── tokio v1.49.0 (*)
│   │   │   └── tracing v0.1.44 (*)
│   │   ├── actix-service v2.0.3 (*)
│   │   ├── actix-utils v3.0.1 (*)
│   │   ├── actix-web-codegen v4.3.0 (proc-macro)
│   │   │   ├── actix-router v0.5.3
│   │   │   │   ├── bytestring v1.5.0
│   │   │   │   │   └── bytes v1.11.0
│   │   │   │   ├── cfg-if v1.0.4
│   │   │   │   ├── regex-lite v0.1.8
│   │   │   │   ├── serde v1.0.228
│   │   │   │   │   └── serde_core v1.0.228
│   │   │   │   └── tracing v0.1.44
│   │   │   │       ├── log v0.4.29
│   │   │   │       ├── pin-project-lite v0.2.16
│   │   │   │       └── tracing-core v0.1.36
│   │   │   ├── proc-macro2 v1.0.106 (*)
│   │   │   ├── quote v1.0.44 (*)
│   │   │   └── syn v2.0.114 (*)
│   │   ├── bytes v1.11.0
│   │   ├── bytestring v1.5.0 (*)
│   │   ├── cfg-if v1.0.4
│   │   ├── cookie v0.16.2
│   │   │   ├── percent-encoding v2.3.2
│   │   │   └── time v0.3.46
│   │   │       ├── deranged v0.5.5
│   │   │       │   └── powerfmt v0.2.0
│   │   │       ├── itoa v1.0.17
│   │   │       ├── num-conv v0.2.0
│   │   │       ├── powerfmt v0.2.0
│   │   │       ├── time-core v0.1.8
│   │   │       └── time-macros v0.2.26 (proc-macro)
│   │   │           ├── num-conv v0.2.0
│   │   │           └── time-core v0.1.8
│   │   │   [build-dependencies]
│   │   │   └── version_check v0.9.5
│   │   ├── derive_more v2.1.1 (*)
│   │   ├── encoding_rs v0.8.35 (*)
│   │   ├── foldhash v0.1.5
│   │   ├── futures-core v0.3.31
│   │   ├── futures-util v0.3.31 (*)
│   │   ├── impl-more v0.1.9
│   │   ├── itoa v1.0.17
│   │   ├── language-tags v0.3.2
│   │   ├── log v0.4.29
│   │   ├── mime v0.3.17
│   │   ├── once_cell v1.21.3
│   │   ├── pin-project-lite v0.2.16
│   │   ├── regex-lite v0.1.8
│   │   ├── serde v1.0.228 (*)
│   │   ├── serde_json v1.0.149
│   │   │   ├── itoa v1.0.17
│   │   │   ├── memchr v2.7.6
│   │   │   ├── serde_core v1.0.228
│   │   │   └── zmij v1.0.16
│   │   ├── serde_urlencoded v0.7.1
│   │   │   ├── form_urlencoded v1.2.2
│   │   │   │   └── percent-encoding v2.3.2
│   │   │   ├── itoa v1.0.17
│   │   │   ├── ryu v1.0.22
│   │   │   └── serde v1.0.228 (*)
│   │   ├── smallvec v1.15.1
│   │   ├── socket2 v0.6.2 (*)
│   │   ├── time v0.3.46 (*)
│   │   ├── tracing v0.1.44 (*)
│   │   └── url v2.5.8
│   │       ├── form_urlencoded v1.2.2 (*)
│   │       ├── idna v1.1.0
│   │       │   ├── idna_adapter v1.2.1
│   │       │   │   ├── icu_normalizer v2.1.1
│   │       │   │   │   ├── icu_collections v2.1.1
│   │       │   │   │   │   ├── displaydoc v0.2.5 (proc-macro)
│   │       │   │   │   │   │   ├── proc-macro2 v1.0.106 (*)
│   │       │   │   │   │   │   ├── quote v1.0.44 (*)
│   │       │   │   │   │   │   └── syn v2.0.114 (*)
│   │       │   │   │   │   ├── potential_utf v0.1.4
│   │       │   │   │   │   │   └── zerovec v0.11.5
│   │       │   │   │   │   │       ├── yoke v0.8.1
│   │       │   │   │   │   │       │   ├── stable_deref_trait v1.2.1
│   │       │   │   │   │   │       │   ├── yoke-derive v0.8.1 (proc-macro)
│   │       │   │   │   │   │       │   │   ├── proc-macro2 v1.0.106 (*)
│   │       │   │   │   │   │       │   │   ├── quote v1.0.44 (*)
│   │       │   │   │   │   │       │   │   ├── syn v2.0.114 (*)
│   │       │   │   │   │   │       │   │   └── synstructure v0.13.2
│   │       │   │   │   │   │       │   │       ├── proc-macro2 v1.0.106 (*)
│   │       │   │   │   │   │       │   │       ├── quote v1.0.44 (*)
│   │       │   │   │   │   │       │   │       └── syn v2.0.114 (*)
│   │       │   │   │   │   │       │   └── zerofrom v0.1.6
│   │       │   │   │   │   │       │       └── zerofrom-derive v0.1.6 (proc-macro)
│   │       │   │   │   │   │       │           ├── proc-macro2 v1.0.106 (*)
│   │       │   │   │   │   │       │           ├── quote v1.0.44 (*)
│   │       │   │   │   │   │       │           ├── syn v2.0.114 (*)
│   │       │   │   │   │   │       │           └── synstructure v0.13.2 (*)
│   │       │   │   │   │   │       ├── zerofrom v0.1.6 (*)
│   │       │   │   │   │   │       └── zerovec-derive v0.11.2 (proc-macro)
│   │       │   │   │   │   │           ├── proc-macro2 v1.0.106 (*)
│   │       │   │   │   │   │           ├── quote v1.0.44 (*)
│   │       │   │   │   │   │           └── syn v2.0.114 (*)
│   │       │   │   │   │   ├── yoke v0.8.1 (*)
│   │       │   │   │   │   ├── zerofrom v0.1.6 (*)
│   │       │   │   │   │   └── zerovec v0.11.5 (*)
│   │       │   │   │   ├── icu_normalizer_data v2.1.1
│   │       │   │   │   ├── icu_provider v2.1.1
│   │       │   │   │   │   ├── displaydoc v0.2.5 (proc-macro) (*)
│   │       │   │   │   │   ├── icu_locale_core v2.1.1
│   │       │   │   │   │   │   ├── displaydoc v0.2.5 (proc-macro) (*)
│   │       │   │   │   │   │   ├── litemap v0.8.1
│   │       │   │   │   │   │   ├── tinystr v0.8.2
│   │       │   │   │   │   │   │   ├── displaydoc v0.2.5 (proc-macro) (*)
│   │       │   │   │   │   │   │   └── zerovec v0.11.5 (*)
│   │       │   │   │   │   │   ├── writeable v0.6.2
│   │       │   │   │   │   │   └── zerovec v0.11.5 (*)
│   │       │   │   │   │   ├── writeable v0.6.2
│   │       │   │   │   │   ├── yoke v0.8.1 (*)
│   │       │   │   │   │   ├── zerofrom v0.1.6 (*)
│   │       │   │   │   │   ├── zerotrie v0.2.3
│   │       │   │   │   │   │   ├── displaydoc v0.2.5 (proc-macro) (*)
│   │       │   │   │   │   │   ├── yoke v0.8.1 (*)
│   │       │   │   │   │   │   └── zerofrom v0.1.6 (*)
│   │       │   │   │   │   └── zerovec v0.11.5 (*)
│   │       │   │   │   ├── smallvec v1.15.1
│   │       │   │   │   └── zerovec v0.11.5 (*)
│   │       │   │   └── icu_properties v2.1.2
│   │       │   │       ├── icu_collections v2.1.1 (*)
│   │       │   │       ├── icu_locale_core v2.1.1 (*)
│   │       │   │       ├── icu_properties_data v2.1.2
│   │       │   │       ├── icu_provider v2.1.1 (*)
│   │       │   │       ├── zerotrie v0.2.3 (*)
│   │       │   │       └── zerovec v0.11.5 (*)
│   │       │   ├── smallvec v1.15.1
│   │       │   └── utf8_iter v1.0.4
│   │       └── percent-encoding v2.3.2
│   ├── bitflags v2.10.0
│   ├── bytes v1.11.0
│   ├── derive_more v2.1.1 (*)
│   ├── futures-core v0.3.31
│   ├── http-range v0.1.5
│   ├── log v0.4.29
│   ├── mime v0.3.17
│   ├── mime_guess v2.0.5
│   │   ├── mime v0.3.17
│   │   └── unicase v2.9.0
│   │   [build-dependencies]
│   │   └── unicase v2.9.0
│   ├── percent-encoding v2.3.2
│   ├── pin-project-lite v0.2.16
│   └── v_htmlescape v0.15.8
├── actix-multipart v0.7.2
│   ├── actix-multipart-derive v0.7.0 (proc-macro)
│   │   ├── darling v0.20.11
│   │   │   ├── darling_core v0.20.11
│   │   │   │   ├── fnv v1.0.7
│   │   │   │   ├── ident_case v1.0.1
│   │   │   │   ├── proc-macro2 v1.0.106 (*)
│   │   │   │   ├── quote v1.0.44 (*)
│   │   │   │   ├── strsim v0.11.1
│   │   │   │   └── syn v2.0.114 (*)
│   │   │   └── darling_macro v0.20.11 (proc-macro)
│   │   │       ├── darling_core v0.20.11 (*)
│   │   │       ├── quote v1.0.44 (*)
│   │   │       └── syn v2.0.114 (*)
│   │   ├── parse-size v1.1.0
│   │   ├── proc-macro2 v1.0.106 (*)
│   │   ├── quote v1.0.44 (*)
│   │   └── syn v2.0.114 (*)
│   ├── actix-utils v3.0.1 (*)
│   ├── actix-web v4.12.1 (*)
│   ├── derive_more v0.99.20 (proc-macro)
│   │   ├── convert_case v0.4.0
│   │   ├── proc-macro2 v1.0.106 (*)
│   │   ├── quote v1.0.44 (*)
│   │   └── syn v2.0.114 (*)
│   │   [build-dependencies]
│   │   └── rustc_version v0.4.1 (*)
│   ├── futures-core v0.3.31
│   ├── futures-util v0.3.31 (*)
│   ├── httparse v1.10.1
│   ├── local-waker v0.1.4
│   ├── log v0.4.29
│   ├── memchr v2.7.6
│   ├── mime v0.3.17
│   ├── rand v0.8.5
│   │   ├── libc v0.2.180
│   │   ├── rand_chacha v0.3.1
│   │   │   ├── ppv-lite86 v0.2.21
│   │   │   │   └── zerocopy v0.8.33
│   │   │   └── rand_core v0.6.4
│   │   │       └── getrandom v0.2.17
│   │   │           ├── cfg-if v1.0.4
│   │   │           └── libc v0.2.180
│   │   └── rand_core v0.6.4 (*)
│   ├── serde v1.0.228 (*)
│   ├── serde_json v1.0.149 (*)
│   ├── serde_plain v1.0.2
│   │   └── serde v1.0.228 (*)
│   └── tokio v1.49.0 (*)
├── actix-web v4.12.1 (*)
├── aes-gcm v0.10.3
│   ├── aead v0.5.2
│   │   ├── crypto-common v0.1.7
│   │   │   ├── generic-array v0.14.7
│   │   │   │   ├── typenum v1.19.0
│   │   │   │   └── zeroize v1.8.2
│   │   │   │   [build-dependencies]
│   │   │   │   └── version_check v0.9.5
│   │   │   └── typenum v1.19.0
│   │   └── generic-array v0.14.7 (*)
│   ├── aes v0.8.4
│   │   ├── cfg-if v1.0.4
│   │   ├── cipher v0.4.4
│   │   │   ├── crypto-common v0.1.7 (*)
│   │   │   └── inout v0.1.4
│   │   │       └── generic-array v0.14.7 (*)
│   │   └── cpufeatures v0.2.17
│   ├── cipher v0.4.4 (*)
│   ├── ctr v0.9.2
│   │   └── cipher v0.4.4 (*)
│   ├── ghash v0.5.1
│   │   ├── opaque-debug v0.3.1
│   │   └── polyval v0.6.2
│   │       ├── cfg-if v1.0.4
│   │       ├── cpufeatures v0.2.17
│   │       ├── opaque-debug v0.3.1
│   │       └── universal-hash v0.5.1
│   │           ├── crypto-common v0.1.7 (*)
│   │           └── subtle v2.6.1
│   └── subtle v2.6.1
├── argon2 v0.5.3
│   ├── base64ct v1.8.3
│   ├── blake2 v0.10.6
│   │   └── digest v0.10.7
│   │       ├── block-buffer v0.10.4
│   │       │   └── generic-array v0.14.7 (*)
│   │       ├── const-oid v0.9.6
│   │       ├── crypto-common v0.1.7 (*)
│   │       └── subtle v2.6.1
│   ├── cpufeatures v0.2.17
│   └── password-hash v0.5.0
│       ├── base64ct v1.8.3
│       ├── rand_core v0.6.4 (*)
│       └── subtle v2.6.1
├── async-trait v0.1.89 (proc-macro)
│   ├── proc-macro2 v1.0.106 (*)
│   ├── quote v1.0.44 (*)
│   └── syn v2.0.114 (*)
├── base64 v0.22.1
├── chrono v0.4.43
│   ├── iana-time-zone v0.1.65
│   ├── num-traits v0.2.19
│   │   └── libm v0.2.16
│   │   [build-dependencies]
│   │   └── autocfg v1.5.0
│   └── serde v1.0.228 (*)
├── clap v4.5.56
│   ├── clap_builder v4.5.56
│   │   ├── anstyle v1.0.13
│   │   └── clap_lex v0.7.7
│   └── clap_derive v4.5.55 (proc-macro)
│       ├── heck v0.5.0
│       ├── proc-macro2 v1.0.106 (*)
│       ├── quote v1.0.44 (*)
│       └── syn v2.0.114 (*)
├── ecdsa v0.16.9
│   ├── der v0.7.10
│   │   ├── const-oid v0.9.6
│   │   ├── pem-rfc7468 v0.7.0
│   │   │   └── base64ct v1.8.3
│   │   └── zeroize v1.8.2
│   ├── digest v0.10.7 (*)
│   ├── elliptic-curve v0.13.8
│   │   ├── base16ct v0.2.0
│   │   ├── crypto-bigint v0.5.5
│   │   │   ├── generic-array v0.14.7 (*)
│   │   │   ├── rand_core v0.6.4 (*)
│   │   │   ├── subtle v2.6.1
│   │   │   └── zeroize v1.8.2
│   │   ├── digest v0.10.7 (*)
│   │   ├── ff v0.13.1
│   │   │   ├── rand_core v0.6.4 (*)
│   │   │   └── subtle v2.6.1
│   │   ├── generic-array v0.14.7 (*)
│   │   ├── group v0.13.0
│   │   │   ├── ff v0.13.1 (*)
│   │   │   ├── rand_core v0.6.4 (*)
│   │   │   └── subtle v2.6.1
│   │   ├── hkdf v0.12.4
│   │   │   └── hmac v0.12.1
│   │   │       └── digest v0.10.7 (*)
│   │   ├── pem-rfc7468 v0.7.0 (*)
│   │   ├── pkcs8 v0.10.2
│   │   │   ├── der v0.7.10 (*)
│   │   │   └── spki v0.7.3
│   │   │       └── der v0.7.10 (*)
│   │   ├── rand_core v0.6.4 (*)
│   │   ├── sec1 v0.7.3
│   │   │   ├── base16ct v0.2.0
│   │   │   ├── der v0.7.10 (*)
│   │   │   ├── generic-array v0.14.7 (*)
│   │   │   ├── pkcs8 v0.10.2 (*)
│   │   │   ├── subtle v2.6.1
│   │   │   └── zeroize v1.8.2
│   │   ├── subtle v2.6.1
│   │   └── zeroize v1.8.2
│   ├── rfc6979 v0.4.0
│   │   ├── hmac v0.12.1 (*)
│   │   └── subtle v2.6.1
│   ├── signature v2.2.0
│   │   ├── digest v0.10.7 (*)
│   │   └── rand_core v0.6.4 (*)
│   └── spki v0.7.3 (*)
├── elliptic-curve v0.13.8 (*)
├── futures-util v0.3.31 (*)
├── hex v0.4.3
├── id3 v0.6.6
│   ├── bitflags v1.3.2
│   ├── byteorder v1.5.0
│   └── flate2 v1.1.8
│       ├── crc32fast v1.5.0
│       │   └── cfg-if v1.0.4
│       └── miniz_oxide v0.8.9
│           ├── adler2 v2.0.1
│           └── simd-adler32 v0.3.8
├── jsonwebtoken v10.3.0
│   ├── base64 v0.22.1
│   ├── ed25519-dalek v2.2.0
│   │   ├── curve25519-dalek v4.1.3
│   │   │   ├── cfg-if v1.0.4
│   │   │   ├── cpufeatures v0.2.17
│   │   │   ├── curve25519-dalek-derive v0.1.1 (proc-macro)
│   │   │   │   ├── proc-macro2 v1.0.106 (*)
│   │   │   │   ├── quote v1.0.44 (*)
│   │   │   │   └── syn v2.0.114 (*)
│   │   │   ├── digest v0.10.7 (*)
│   │   │   ├── subtle v2.6.1
│   │   │   └── zeroize v1.8.2
│   │   │   [build-dependencies]
│   │   │   └── rustc_version v0.4.1 (*)
│   │   ├── ed25519 v2.2.3
│   │   │   ├── pkcs8 v0.10.2 (*)
│   │   │   └── signature v2.2.0 (*)
│   │   ├── sha2 v0.10.9
│   │   │   ├── cfg-if v1.0.4
│   │   │   ├── cpufeatures v0.2.17
│   │   │   └── digest v0.10.7 (*)
│   │   ├── subtle v2.6.1
│   │   └── zeroize v1.8.2
│   ├── hmac v0.12.1 (*)
│   ├── p256 v0.13.2
│   │   ├── ecdsa v0.16.9 (*)
│   │   ├── elliptic-curve v0.13.8 (*)
│   │   ├── primeorder v0.13.6
│   │   │   └── elliptic-curve v0.13.8 (*)
│   │   └── sha2 v0.10.9 (*)
│   ├── p384 v0.13.1
│   │   ├── ecdsa v0.16.9 (*)
│   │   ├── elliptic-curve v0.13.8 (*)
│   │   ├── primeorder v0.13.6 (*)
│   │   └── sha2 v0.10.9 (*)
│   ├── pem v3.0.6
│   │   └── base64 v0.22.1
│   ├── rand v0.8.5 (*)
│   ├── rsa v0.9.10
│   │   ├── const-oid v0.9.6
│   │   ├── digest v0.10.7 (*)
│   │   ├── num-bigint-dig v0.8.6
│   │   │   ├── lazy_static v1.5.0
│   │   │   │   └── spin v0.9.8
│   │   │   ├── libm v0.2.16
│   │   │   ├── num-integer v0.1.46
│   │   │   │   └── num-traits v0.2.19 (*)
│   │   │   ├── num-iter v0.1.45
│   │   │   │   ├── num-integer v0.1.46 (*)
│   │   │   │   └── num-traits v0.2.19 (*)
│   │   │   │   [build-dependencies]
│   │   │   │   └── autocfg v1.5.0
│   │   │   ├── num-traits v0.2.19 (*)
│   │   │   ├── rand v0.8.5 (*)
│   │   │   ├── smallvec v1.15.1
│   │   │   └── zeroize v1.8.2
│   │   ├── num-integer v0.1.46 (*)
│   │   ├── num-traits v0.2.19 (*)
│   │   ├── pkcs1 v0.7.5
│   │   │   ├── der v0.7.10 (*)
│   │   │   ├── pkcs8 v0.10.2 (*)
│   │   │   └── spki v0.7.3 (*)
│   │   ├── pkcs8 v0.10.2 (*)
│   │   ├── rand_core v0.6.4 (*)
│   │   ├── signature v2.2.0 (*)
│   │   ├── spki v0.7.3 (*)
│   │   ├── subtle v2.6.1
│   │   └── zeroize v1.8.2
│   ├── serde v1.0.228 (*)
│   ├── serde_json v1.0.149 (*)
│   ├── sha2 v0.10.9 (*)
│   ├── signature v2.2.0 (*)
│   └── simple_asn1 v0.6.3
│       ├── num-bigint v0.4.6
│       │   ├── num-integer v0.1.46 (*)
│       │   └── num-traits v0.2.19 (*)
│       ├── num-traits v0.2.19 (*)
│       ├── thiserror v2.0.18
│       │   └── thiserror-impl v2.0.18 (proc-macro)
│       │       ├── proc-macro2 v1.0.106 (*)
│       │       ├── quote v1.0.44 (*)
│       │       └── syn v2.0.114 (*)
│       └── time v0.3.46 (*)
├── lazy_static v1.5.0 (*)
├── maxminddb v0.27.1
│   ├── ipnetwork v0.21.1
│   ├── log v0.4.29
│   ├── memchr v2.7.6
│   ├── serde v1.0.228 (*)
│   └── thiserror v2.0.18 (*)
├── md-5 v0.10.6
│   ├── cfg-if v1.0.4
│   └── digest v0.10.7 (*)
├── memmap2 v0.9.9
│   └── libc v0.2.180
├── metaflac v0.2.8
│   ├── byteorder v1.5.0
│   └── hex v0.4.3
├── mime_guess v2.0.5 (*)
├── once_cell v1.21.3
├── p256 v0.13.2 (*)
├── pulldown-cmark v0.12.2
│   ├── bitflags v2.10.0
│   ├── memchr v2.7.6
│   ├── pulldown-cmark-escape v0.11.0
│   └── unicase v2.9.0
├── r2d2 v0.8.10
│   ├── log v0.4.29
│   ├── parking_lot v0.12.5 (*)
│   └── scheduled-thread-pool v0.2.7
│       └── parking_lot v0.12.5 (*)
├── r2d2_sqlite v0.25.0
│   ├── r2d2 v0.8.10 (*)
│   ├── rusqlite v0.32.1
│   │   ├── bitflags v2.10.0
│   │   ├── chrono v0.4.43 (*)
│   │   ├── fallible-iterator v0.3.0
│   │   ├── fallible-streaming-iterator v0.1.9
│   │   ├── hashlink v0.9.1
│   │   │   └── hashbrown v0.14.5
│   │   │       └── ahash v0.8.12
│   │   │           ├── cfg-if v1.0.4
│   │   │           ├── once_cell v1.21.3
│   │   │           └── zerocopy v0.8.33
│   │   │           [build-dependencies]
│   │   │           └── version_check v0.9.5
│   │   ├── libsqlite3-sys v0.30.1
│   │   │   [build-dependencies]
│   │   │   ├── cc v1.2.55
│   │   │   │   ├── find-msvc-tools v0.1.9
│   │   │   │   └── shlex v1.3.0
│   │   │   ├── pkg-config v0.3.32
│   │   │   └── vcpkg v0.2.15
│   │   ├── serde_json v1.0.149 (*)
│   │   └── smallvec v1.15.1
│   └── uuid v1.19.0
│       ├── getrandom v0.3.4
│       │   ├── cfg-if v1.0.4
│       │   └── libc v0.2.180
│       └── rand v0.9.2
│           ├── rand_chacha v0.9.0
│           │   ├── ppv-lite86 v0.2.21 (*)
│           │   └── rand_core v0.9.5
│           │       └── getrandom v0.3.4 (*)
│           └── rand_core v0.9.5 (*)
├── rand v0.8.5 (*)
├── regex v1.12.2
│   ├── aho-corasick v1.1.4
│   │   └── memchr v2.7.6
│   ├── memchr v2.7.6
│   ├── regex-automata v0.4.13
│   │   ├── aho-corasick v1.1.4 (*)
│   │   ├── memchr v2.7.6
│   │   └── regex-syntax v0.8.8
│   └── regex-syntax v0.8.8
├── reqwest v0.12.28
│   ├── base64 v0.22.1
│   ├── bytes v1.11.0
│   ├── futures-core v0.3.31
│   ├── http v1.4.0
│   │   ├── bytes v1.11.0
│   │   └── itoa v1.0.17
│   ├── http-body v1.0.1
│   │   ├── bytes v1.11.0
│   │   └── http v1.4.0 (*)
│   ├── http-body-util v0.1.3
│   │   ├── bytes v1.11.0
│   │   ├── futures-core v0.3.31
│   │   ├── http v1.4.0 (*)
│   │   ├── http-body v1.0.1 (*)
│   │   └── pin-project-lite v0.2.16
│   ├── hyper v1.8.1
│   │   ├── atomic-waker v1.1.2
│   │   ├── bytes v1.11.0
│   │   ├── futures-channel v0.3.31
│   │   │   └── futures-core v0.3.31
│   │   ├── futures-core v0.3.31
│   │   ├── http v1.4.0 (*)
│   │   ├── http-body v1.0.1 (*)
│   │   ├── httparse v1.10.1
│   │   ├── itoa v1.0.17
│   │   ├── pin-project-lite v0.2.16
│   │   ├── pin-utils v0.1.0
│   │   ├── smallvec v1.15.1
│   │   ├── tokio v1.49.0 (*)
│   │   └── want v0.3.1
│   │       └── try-lock v0.2.5
│   ├── hyper-rustls v0.27.7
│   │   ├── http v1.4.0 (*)
│   │   ├── hyper v1.8.1 (*)
│   │   ├── hyper-util v0.1.19
│   │   │   ├── base64 v0.22.1
│   │   │   ├── bytes v1.11.0
│   │   │   ├── futures-channel v0.3.31 (*)
│   │   │   ├── futures-core v0.3.31
│   │   │   ├── futures-util v0.3.31 (*)
│   │   │   ├── http v1.4.0 (*)
│   │   │   ├── http-body v1.0.1 (*)
│   │   │   ├── hyper v1.8.1 (*)
│   │   │   ├── ipnet v2.11.0
│   │   │   ├── libc v0.2.180
│   │   │   ├── percent-encoding v2.3.2
│   │   │   ├── pin-project-lite v0.2.16
│   │   │   ├── socket2 v0.5.10 (*)
│   │   │   ├── tokio v1.49.0 (*)
│   │   │   ├── tower-service v0.3.3
│   │   │   └── tracing v0.1.44 (*)
│   │   ├── rustls v0.23.36
│   │   │   ├── once_cell v1.21.3
│   │   │   ├── ring v0.17.14
│   │   │   │   ├── cfg-if v1.0.4
│   │   │   │   ├── getrandom v0.2.17 (*)
│   │   │   │   └── untrusted v0.9.0
│   │   │   │   [build-dependencies]
│   │   │   │   └── cc v1.2.55 (*)
│   │   │   ├── rustls-pki-types v1.14.0
│   │   │   │   └── zeroize v1.8.2
│   │   │   ├── rustls-webpki v0.103.9
│   │   │   │   ├── ring v0.17.14 (*)
│   │   │   │   ├── rustls-pki-types v1.14.0 (*)
│   │   │   │   └── untrusted v0.9.0
│   │   │   ├── subtle v2.6.1
│   │   │   └── zeroize v1.8.2
│   │   ├── rustls-pki-types v1.14.0 (*)
│   │   ├── tokio v1.49.0 (*)
│   │   ├── tokio-rustls v0.26.4
│   │   │   ├── rustls v0.23.36 (*)
│   │   │   └── tokio v1.49.0 (*)
│   │   ├── tower-service v0.3.3
│   │   └── webpki-roots v1.0.6
│   │       └── rustls-pki-types v1.14.0 (*)
│   ├── hyper-util v0.1.19 (*)
│   ├── log v0.4.29
│   ├── percent-encoding v2.3.2
│   ├── pin-project-lite v0.2.16
│   ├── rustls v0.23.36 (*)
│   ├── rustls-pki-types v1.14.0 (*)
│   ├── serde v1.0.228 (*)
│   ├── serde_json v1.0.149 (*)
│   ├── serde_urlencoded v0.7.1 (*)
│   ├── sync_wrapper v1.0.2
│   │   └── futures-core v0.3.31
│   ├── tokio v1.49.0 (*)
│   ├── tokio-rustls v0.26.4 (*)
│   ├── tower v0.5.3
│   │   ├── futures-core v0.3.31
│   │   ├── futures-util v0.3.31 (*)
│   │   ├── pin-project-lite v0.2.16
│   │   ├── sync_wrapper v1.0.2 (*)
│   │   ├── tokio v1.49.0 (*)
│   │   ├── tower-layer v0.3.3
│   │   └── tower-service v0.3.3
│   ├── tower-http v0.6.8
│   │   ├── bitflags v2.10.0
│   │   ├── bytes v1.11.0
│   │   ├── futures-util v0.3.31 (*)
│   │   ├── http v1.4.0 (*)
│   │   ├── http-body v1.0.1 (*)
│   │   ├── iri-string v0.7.10
│   │   ├── pin-project-lite v0.2.16
│   │   ├── tower v0.5.3 (*)
│   │   ├── tower-layer v0.3.3
│   │   └── tower-service v0.3.3
│   ├── tower-service v0.3.3
│   ├── url v2.5.8 (*)
│   └── webpki-roots v1.0.6 (*)
├── rusqlite v0.32.1 (*)
├── rust-embed v8.11.0
│   ├── rust-embed-impl v8.11.0 (proc-macro)
│   │   ├── proc-macro2 v1.0.106 (*)
│   │   ├── quote v1.0.44 (*)
│   │   ├── rust-embed-utils v8.11.0
│   │   │   ├── globset v0.4.18
│   │   │   │   ├── aho-corasick v1.1.4
│   │   │   │   │   └── memchr v2.7.6
│   │   │   │   ├── bstr v1.12.1
│   │   │   │   │   └── memchr v2.7.6
│   │   │   │   ├── log v0.4.29
│   │   │   │   ├── regex-automata v0.4.13
│   │   │   │   │   ├── aho-corasick v1.1.4 (*)
│   │   │   │   │   ├── memchr v2.7.6
│   │   │   │   │   └── regex-syntax v0.8.8
│   │   │   │   └── regex-syntax v0.8.8
│   │   │   ├── sha2 v0.10.9
│   │   │   │   ├── cfg-if v1.0.4
│   │   │   │   ├── cpufeatures v0.2.17
│   │   │   │   └── digest v0.10.7
│   │   │   │       ├── block-buffer v0.10.4
│   │   │   │       │   └── generic-array v0.14.7
│   │   │   │       │       └── typenum v1.19.0
│   │   │   │       │       [build-dependencies]
│   │   │   │       │       └── version_check v0.9.5
│   │   │   │       └── crypto-common v0.1.7
│   │   │   │           ├── generic-array v0.14.7 (*)
│   │   │   │           └── typenum v1.19.0
│   │   │   └── walkdir v2.5.0
│   │   │       └── same-file v1.0.6
│   │   ├── syn v2.0.114 (*)
│   │   └── walkdir v2.5.0 (*)
│   ├── rust-embed-utils v8.11.0
│   │   ├── globset v0.4.18
│   │   │   ├── aho-corasick v1.1.4 (*)
│   │   │   ├── bstr v1.12.1
│   │   │   │   └── memchr v2.7.6
│   │   │   ├── log v0.4.29
│   │   │   ├── regex-automata v0.4.13 (*)
│   │   │   └── regex-syntax v0.8.8
│   │   ├── sha2 v0.10.9 (*)
│   │   └── walkdir v2.5.0
│   │       └── same-file v1.0.6
│   └── walkdir v2.5.0 (*)
├── serde v1.0.228 (*)
├── serde_json v1.0.149 (*)
├── sha2 v0.10.9 (*)
├── snowflake-id-generator v0.4.0
│   └── tokio v1.49.0 (*)
├── spki v0.7.3 (*)
├── tera v1.20.1
│   ├── chrono v0.4.43 (*)
│   ├── chrono-tz v0.9.0
│   │   ├── chrono v0.4.43 (*)
│   │   └── phf v0.11.3
│   │       └── phf_shared v0.11.3
│   │           └── siphasher v1.0.2
│   │   [build-dependencies]
│   │   └── chrono-tz-build v0.3.0
│   │       ├── parse-zoneinfo v0.3.1
│   │       │   └── regex v1.12.2
│   │       │       ├── regex-automata v0.4.13 (*)
│   │       │       └── regex-syntax v0.8.8
│   │       ├── phf v0.11.3
│   │       │   └── phf_shared v0.11.3
│   │       │       └── siphasher v1.0.2
│   │       └── phf_codegen v0.11.3
│   │           ├── phf_generator v0.11.3
│   │           │   ├── phf_shared v0.11.3 (*)
│   │           │   └── rand v0.8.5
│   │           │       └── rand_core v0.6.4
│   │           └── phf_shared v0.11.3 (*)
│   ├── globwalk v0.9.1
│   │   ├── bitflags v2.10.0
│   │   ├── ignore v0.4.25
│   │   │   ├── crossbeam-deque v0.8.6
│   │   │   │   ├── crossbeam-epoch v0.9.18
│   │   │   │   │   └── crossbeam-utils v0.8.21
│   │   │   │   └── crossbeam-utils v0.8.21
│   │   │   ├── globset v0.4.18 (*)
│   │   │   ├── log v0.4.29
│   │   │   ├── memchr v2.7.6
│   │   │   ├── regex-automata v0.4.13 (*)
│   │   │   ├── same-file v1.0.6
│   │   │   └── walkdir v2.5.0 (*)
│   │   └── walkdir v2.5.0 (*)
│   ├── humansize v2.1.3
│   │   └── libm v0.2.16
│   ├── lazy_static v1.5.0 (*)
│   ├── percent-encoding v2.3.2
│   ├── pest v2.8.5
│   │   ├── memchr v2.7.6
│   │   └── ucd-trie v0.1.7
│   ├── pest_derive v2.8.5 (proc-macro)
│   │   ├── pest v2.8.5
│   │   │   ├── memchr v2.7.6
│   │   │   └── ucd-trie v0.1.7
│   │   └── pest_generator v2.8.5
│   │       ├── pest v2.8.5 (*)
│   │       ├── pest_meta v2.8.5
│   │       │   └── pest v2.8.5 (*)
│   │       │   [build-dependencies]
│   │       │   └── sha2 v0.10.9 (*)
│   │       ├── proc-macro2 v1.0.106 (*)
│   │       ├── quote v1.0.44 (*)
│   │       └── syn v2.0.114 (*)
│   ├── rand v0.8.5 (*)
│   ├── regex v1.12.2 (*)
│   ├── serde v1.0.228 (*)
│   ├── serde_json v1.0.149 (*)
│   ├── slug v0.1.6
│   │   └── deunicode v1.6.2
│   └── unicode-segmentation v1.12.0
├── tokio v1.49.0 (*)
├── toml v0.8.23
│   ├── serde v1.0.228 (*)
│   ├── serde_spanned v0.6.9
│   │   └── serde v1.0.228 (*)
│   ├── toml_datetime v0.6.11
│   │   └── serde v1.0.228 (*)
│   └── toml_edit v0.22.27
│       ├── indexmap v2.13.0
│       │   ├── equivalent v1.0.2
│       │   └── hashbrown v0.16.1
│       ├── serde v1.0.228 (*)
│       ├── serde_spanned v0.6.9 (*)
│       ├── toml_datetime v0.6.11 (*)
│       └── winnow v0.7.14
└── urlencoding v2.1.3

```
