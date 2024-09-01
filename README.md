# 说明
利用Rust编写的高效URL测活工具，主要特点快速、批量、轻量

# 用法
## 帮助信息
```text
An efficient and fast url survival detection tool

Usage: windfire [OPTIONS]

Options:
  -t, --thread <THREAD>            Setting the number of threads [default: 50]
  -u, --url <URL>                  Enter an url
  -f, --file <FILE>                Enter a file path
  -s, --timeout <TIMEOUT>          The http request timeout [default: 10]
  -c, --status-code <STATUS_CODE>  Display the specified status code [default: 200]
  -p, --path <PATH>                Designated path scan [default: ]
  -h, --help                       Print help (see more with '--help')
  -V, --version                    Print version
```
## 参数说明
* -t --thread 设置线程数量，默认50
* -u --url 输入一个url
* -f --file 输入一个文件路径，文件内每行一个url，txt文本
* -s --timeout 设置http请求超时时间，默认10秒
* -c --status-code 显示指定的状态码，默认200，可以输入多个，用逗号隔开，如200,403
* -p --path 指定扫描路径，默认为空，不指定，如 -p admin
* -h --help 显示帮助信息
* -V --version 显示版本信息

## 使用
1. 单个目标指定
```shell
windfire -u https://www.baidu.com
```
2. 批量执行目标
```shell
windfire -f urls.txt
```
3. 指定路径测活
```
windfire -f urls.txt -p admin -c 200
```
## 默认打印信息
```shell
https://www.baidu.com [200] [百度一下，你就知道] [BWS/1.1] [https://www.baidu.com/] [414219]
```
包括：起始地址（url）、状态码（status_code）、标题（title）、服务器（server）、跳转后地址（jump_url）、响应页面大小（content_length）