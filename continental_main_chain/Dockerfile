# ベースイメージとして最新のRustを使用
FROM rust:latest

# 作業ディレクトリを設定
WORKDIR /usr/src/app

# 必要なパッケージをインストール
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    openssl \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# ソースコードをコピー
COPY . .

# NTRUクレートの依存関係をビルド
RUN mkdir -p /usr/src/app/ntru
COPY ntru /usr/src/app/ntru
RUN cd /usr/src/app/ntru && cargo build --release

# メインアプリケーションの依存関係をビルド
RUN cargo build --release

# デフォルトの起動コマンド（必要に応じて変更可能）
CMD ["./target/release/continental_main_chain"]
