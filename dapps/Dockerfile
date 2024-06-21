# dapps/Dockerfile

# ベースイメージを指定
FROM python:3.8

# 作業ディレクトリを設定
WORKDIR /usr/src/app

# 必要なパッケージをコピーしてインストール
COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

# アプリケーションのソースコードをコピー
COPY . .

# Flaskアプリケーションの起動
CMD ["python", "app.py"]
