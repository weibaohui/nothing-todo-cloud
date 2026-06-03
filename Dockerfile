FROM rust:1.75-slim AS builder

WORKDIR /app

# 安装编译依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 复制代码
COPY backend/ ./backend/

WORKDIR /app/backend

# 构建后端
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 复制二进制文件
COPY --from=builder /app/backend/target/release/ntd-cloud-server ./

# 复制默认配置
COPY backend/config.yaml ./

# 创建数据目录
RUN mkdir -p /app/data

EXPOSE 8089

ENV RUST_LOG=info

CMD ["./ntd-cloud-server"]
