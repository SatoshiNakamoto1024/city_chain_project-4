# ベースイメージを指定
FROM rust:latest

# 作業ディレクトリを設定
WORKDIR /usr/src/app

# ソースコードをコピー
COPY . .

# 必要なパッケージをインストール
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# NTRUクレートをローカルに追加するためのステップ
COPY ../ntru /usr/src/ntru

# 環境変数を設定してローカルクレートへのパスを指定
ENV RUSTFLAGS="--cfg ntru"
RUN cargo build --release

# アプリケーションの実行
CMD ["./target/release/global_main_chain"]
