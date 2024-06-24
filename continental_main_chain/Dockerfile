FROM rust:latest
WORKDIR /usr/src/app

# 必要なパッケージをインストール
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# ソースコードをコピー
COPY . .

# NTRUクレートの依存関係をビルド
RUN mkdir -p /usr/src/app/ntru
COPY path_to_your_ntru_crate /usr/src/app/ntru
RUN cd /usr/src/app/ntru && cargo build --release

# メインアプリケーションをビルド
RUN cargo build --release

# アプリケーションを起動
CMD ["./target/release/continental_main_chain"]
