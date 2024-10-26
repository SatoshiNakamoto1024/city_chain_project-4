# ベースイメージとして公式のRustランタイムを使用
FROM rust:latest AS builder

# 作業ディレクトリを設定
WORKDIR /usr/src/app

# 必要なパッケージをインストール
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# 依存関係をコンテナにコピー
COPY Cargo.toml Cargo.lock ./

# 依存関係をビルド（最初にダミーファイルでビルド）
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# ソースコードをコピー
COPY . .

# メインアプリケーションのビルド
RUN cargo build --release

# 実行ステージとして軽量なベースイメージを使用
FROM debian:buster-slim

# 必要なライブラリをコピー
COPY --from=builder /usr/src/app/target/release/global_main_chain /usr/local/bin/global_main_chain

# 必要なSSLライブラリをインストール
RUN apt-get update && apt-get install -y \
    libssl-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# 実行可能ファイルをエントリーポイントとして設定
CMD ["/usr/local/bin/global_main_chain"]
