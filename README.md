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
1.集成**常用**功能(文章,评论,后台,附件)，无须也不必要引入任何插件系统\n
2.sqlite文件类型数据库,无须任何其他后台服务\n
3.嵌入式模板(rust-embed)只释放可变的文件目录\n
### 六.致谢
感谢一下开源库的作者的贡献
