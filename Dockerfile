FROM rust:1.75-slim as builder
WORKDIR /app
COPY . .
# 强制执行编译并显示错误详情
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/tvb-proxy /usr/local/bin/tvb-proxy
ENV KEY="2d2fd7b1661b1e28de38268872b48480"
EXPOSE 8080
CMD ["tvb-proxy"]
