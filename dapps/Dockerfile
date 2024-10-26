# ベースイメージを指定（バックエンド用）
FROM python:3.9-slim AS backend

# 作業ディレクトリを設定
WORKDIR /usr/src/app

# 必要なパッケージをコピーしてインストール
COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

# 必要なパッケージをインストール
RUN apt-get update && apt-get install -y \
    gcc \
    libssl-dev \
    git \
    cargo \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# city_chain_projectリポジトリからntruフォルダーをクローン
RUN git clone https://github.com/SatoshiNakamoto1024/city_chain_project-2.git /usr/src/city_chain_project && \
    cp -r /usr/src/city_chain_project/ntru /usr/src/app/ntru

# NTRUクレートの依存関係をコピーしてビルド
RUN cd /usr/src/app/ntru && cargo build --release

# アプリケーションのソースコードをコピー
COPY . .

# フロントエンド用のNode.jsイメージ
FROM node:14 AS frontend

# 作業ディレクトリを設定
WORKDIR /usr/src/app

# フロントエンドの依存関係をインストール
COPY dapps/package.json dapps/package-lock.json ./
RUN npm install --no-cache

# フロントエンドのソースコードをコピーしてビルド
COPY dapps/public ./public
COPY dapps/src ./src
RUN npm run build

# フロントエンドとバックエンドを統合
FROM backend AS final

# ビルドされたフロントエンドのファイルをコピー
COPY --from=frontend /usr/src/app/build /usr/src/app/static

# Flaskアプリケーションの起動
CMD ["python", "app.py"]
