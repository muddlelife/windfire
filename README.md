# Description

A high-performance URL liveness detection tool written in Rust.
Designed to be fast, lightweight, and highly concurrent, with full asynchronous support.

# 中文文档
👉 [中文说明](./zh-CN/README.md)

# Features
| Feature                  | Description                                                      |
| ------------------------ | ---------------------------------------------------------------- |
| Asset Liveness Detection | Check the availability of assets such as domains, IPs, and URLs  |
| Proxy Support            | Supports HTTP, HTTPS, and SOCKS proxies                          |
| Status Code Filtering    | Filter HTTP responses by status code (default: only 200)         |
| Custom Path              | Supports specifying a custom path (default: `/`)                 |
| Fingerprint Detection    | Identifies website CMS using an open-source fingerprint database |
| Layered Scan Mode        | `alive` mode for liveness-only and `full` mode for fingerprinting |
| Scan Rate Control        | `--rate` controls requests per second to protect targets         |
| High Concurrency         | Supports high concurrency and asynchronous execution             |

# Usage

## Help
```text
An efficient and fast url survival detection tool

Usage: windfire [OPTIONS]

Options:
An efficient and fast url survival detection tool

Usage: windfire.exe [OPTIONS] <--url <URL>|--file <FILE>>

Options:
  -t, --thread <THREAD>            Setting the number of threads [default: 50]
  -u, --url <URL>                  Enter a target
  -f, --file <FILE>                Enter a file path
  -s, --timeout <TIMEOUT>          The http request timeout [default: 10]
  -c, --status-code <STATUS_CODE>  Display the specified status code
  -p, --path <PATH>                Designated path scan [default: ]
  -x, --proxy <PROXY>              Supported Proxy socks5, http, and https
  -o, --output <OUTPUT>            Output to CSV or JSON file
      --mode <MODE>                Scan mode: alive or full [default: full]
      --rate <RATE>                Max requests per second (0 means unlimited) [default: 0]
  -h, --help                       Print help
  -V, --version                    Print version
```

## Parameter Description
* -t, --thread Set the number of threads (default: 50)
* -u, --url Specify a single target. Supports IP, IP range, domain, URL, or host:port.
Defaults to scanning ports 80 and 443.
* -f, --file Specify a file containing targets (one per line)
* -s, --timeout Set HTTP request timeout (default: 10 seconds)
* -c, --status-code Filter by specific HTTP status codes.
Multiple values can be provided, separated by commas (e.g. 200,403)
* -p, --path Specify a path to scan (default: empty)
* -x, --proxy Proxy support (socks5, http, https), e.g. socks5://127.0.0.1:1080
* -o, --output Export results to a CSV or JSON file, e.g. -o result.csv or -o result.json
* --mode Scan mode: `alive` (liveness only) or `full` (liveness + fingerprint), default `full`
* --rate Max requests per second. `0` means unlimited (default)
* -h, --help Display help information
* -V, --version Show version information

## Usage Examples

1. Single target
```shell
windfire -u https://www.baidu.com
```
2. Batch scan from file
```shell
windfire -f urls.txt
```
3. Scan with a specific path
```shell
windfire -f urls.txt -p admin -c 200
```
4. Batch scan with output
```shell
windfire -f urls.txt > result.txt
```
5. Use proxy
```shell
windfire -f urls.txt -x socks5://127.0.0.1:1080
```
6. Export results to CSV or JSON
```shell
windfire -f urls.txt -o result.csv
```

## Default Output Format
```text
https://www.baidu.com [200] [百度一下，你就知道] [BWS/1.1] [414219] ["CMS"]
```
Output fields include:
* URL
* Status Code
* Page Title
* Server
* Response Size
* Fingerprint Information

## Release

### v0.5.0
- Added layered scan mode: `--mode alive|full`.
- Added global request rate control: `--rate`.
- Kept compatibility with existing output and filtering parameters.
