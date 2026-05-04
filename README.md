# screenshot-service

一个使用 Rust + Axum + Chromiumoxide 重写的网页截图微服务。

## 接口

- `GET /health`: 健康检查，返回 `{"status":"ok"}`
- `GET /screenshot`: 通过 query 参数截图
- `POST /screenshot`: 通过 JSON body 截图

## GET 请求示例

```bash
# 基本截图
curl "http://localhost:8080/screenshot?url=https://www.google.com"

# 指定尺寸和格式
curl "http://localhost:8080/screenshot?url=https://www.google.com&width=1280&height=720&format=webp&quality=85"

# 完整参数
curl "http://localhost:8080/screenshot?url=https://www.baidu.com&width=1920&height=1080&format=jpeg&quality=90&wait_time=2000&full_page=true&mobile=false"

# 带请求头
curl "http://localhost:8080/screenshot?url=https://example.com&headers=%7B%22Authorization%22%3A%22Bearer%20token123%22%7D"

# 保存到文件
curl -o screenshot.png "http://localhost:8080/screenshot?url=https://www.google.com&format=png"
```

## POST 请求示例

```bash
# 基本 POST 请求
curl -X POST http://localhost:8080/screenshot \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://www.google.com",
    "width": 1920,
    "height": 1080,
    "format": "png"
  }' --output screenshot.png

# 完整参数 POST 请求
curl -X POST http://localhost:8080/screenshot \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://www.example.com",
    "width": 1440,
    "height": 900,
    "format": "webp",
    "quality": 85,
    "wait_time": 3000,
    "wait_for": "#main-content",
    "full_page": false,
    "headers": {
      "Authorization": "Bearer your-token",
      "Cookie": "session=abc123"
    },
    "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
    "device_scale": 2.0,
    "mobile": false,
    "landscape": false,
    "timeout": 60
  }' --output screenshot.webp

# 裁剪区域截图
curl -X POST http://localhost:8080/screenshot \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://www.google.com",
    "clip": {
      "x": 100,
      "y": 100,
      "width": 500,
      "height": 300
    },
    "format": "jpeg",
    "quality": 95
  }' --output cropped.jpg
```

## 参数说明

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `url` | string | 必填 | 目标网页 URL |
| `width` | int | 1920 | 视口宽度 (100-4096) |
| `height` | int | 1080 | 视口高度 (100-10000) |
| `format` | string | png | 输出格式: png/jpeg/jpg/webp |
| `quality` | int | 90 | 压缩质量 (1-100)，用于 jpeg/webp |
| `wait_time` | int | 0 | 额外等待时间，单位毫秒 |
| `wait_for` | string | - | 等待元素出现并可见的 CSS 选择器 |
| `full_page` | bool | false | 是否全页面截图，最大高度 16384px |
| `headers` | object | - | 自定义请求头；GET 时传 JSON 字符串 |
| `user_agent` | string | - | 自定义 User-Agent |
| `device_scale` | float | 1.0 | 设备像素比 |
| `mobile` | bool | false | 移动端模拟 |
| `landscape` | bool | false | 横屏模式 |
| `timeout` | int | 30 | 超时时间，单位秒，最大 120 |
| `clip` | object | - | 裁剪区域 `{x,y,width,height}` |

## 本地运行

本机运行需要安装 Chrome/Chromium。服务会自动探测浏览器，也可以通过 `CHROME_BIN` 指定可执行文件路径。

```bash
export CHROME_BIN=/path/to/chromium
cargo run
curl -o test.png "http://localhost:8080/screenshot?url=https://www.google.com"
```

## Docker 部署

```bash
docker-compose up -d --build
docker-compose logs -f
curl -o test.png "http://localhost:8080/screenshot?url=https://www.google.com"
```
