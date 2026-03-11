# 使用最新的 Rust 镜像以支持最新的依赖包
FROM rust:latest as builder
WORKDIR /app
COPY . .
# 执行编译
RUN cargo build --release

# 运行阶段使用较小的镜像
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
# 将编译好的二进制文件复制过来
COPY --from=builder /app/target/release/tvb-proxy /usr/local/bin/tvb-proxy
# 默认 Key
ENV KEY="2d2fd7b1661b1e28de38268872b48480"
EXPOSE 8080
CMD ["tvb-proxy"]
