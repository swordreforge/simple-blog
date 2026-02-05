# simple-blog
## 也许是最小构建包体的动态博客,但功能全面
### 一.包体小于10M(8.96M)
### 二.运行内存极低,100多m运存即可运行，峰值运行不超过100M，512m服务器即可正常运行
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
### 三.无畏并发
### 四.零插件依赖
### 五.单文件部署
