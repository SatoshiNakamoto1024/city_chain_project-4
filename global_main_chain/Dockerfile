FROM rust:latest

# 作業ディレクトリを設定
WORKDIR /usr/src/app

# 必要なパッケージをインストール
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# ソースコードをコピー
COPY . .

# NTRUクレートの依存関係をコピーしてビルド
COPY ../ntru /usr/src/app/ntru
RUN cd /usr/src/app/ntru && cargo build --release

# メインアプリケーションをビルド
RUN cargo build --release

# アプリケーションを起動
CMD ["./target/release/global_main_chain"]
