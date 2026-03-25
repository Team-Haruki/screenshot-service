# 构建阶段
FROM golang:1.26.1-alpine AS builder

WORKDIR /app

# 安装构建依赖
RUN apk add --no-cache git ca-certificates

# 优先复制依赖清单，利用 Docker 层缓存
COPY go.mod go.sum ./
RUN go mod download

# 复制源代码并构建
COPY *.go ./
RUN CGO_ENABLED=0 GOOS=linux go build -a -ldflags="-w -s" -o /screenshot-service .

# 运行阶段
# 使用 latest 或 edge 以匹配最新的 Chrome 特性
FROM alpine:latest

# 安装 Chromium 和必要的字体
RUN apk add --no-cache \
    chromium \
    nss \
    freetype \
    harfbuzz \
    ca-certificates \
    ttf-freefont \
    font-noto-cjk \
    font-noto-emoji \
    dumb-init \
    && rm -rf /var/cache/apk/*

# 设置 Chrome 环境变量
ENV CHROME_BIN=/usr/bin/chromium-browser \
    CHROME_PATH=/usr/lib/chromium/ \
    CHROMIUM_FLAGS="--disable-software-rasterizer --disable-dev-shm-usage"

# 创建非 root 用户
RUN addgroup -g 1000 appgroup && \
    adduser -u 1000 -G appgroup -s /bin/sh -D appuser

# 创建必要的目录
RUN mkdir -p /tmp/chrome-data && \
    chown -R appuser:appgroup /tmp/chrome-data

WORKDIR /app

# 复制二进制文件
COPY --from=builder /screenshot-service .

# 设置权限
RUN chown -R appuser:appgroup /app

# 切换用户
USER appuser

# 暴露端口
EXPOSE 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# 使用 dumb-init 启动
ENTRYPOINT ["/usr/bin/dumb-init", "--"]
CMD ["./screenshot-service"]