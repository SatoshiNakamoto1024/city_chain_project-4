# ベースイメージを指定
FROM python:3.8

# 作業ディレクトリを設定
WORKDIR /usr/src/app

# 必要なパッケージをコピーしてインストール
COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt

# ntruライブラリをクローンしてインストール
RUN apt-get update && apt-get install -y git
RUN git clone https://github.com/SatoshiNakamoto1024/city_chain_project-2.git /usr/src/ntru
RUN cd /usr/src/ntru && python setup.py install

# アプリケーションのソースコードをコピー
COPY . .

# Flaskアプリケーションの起動
CMD ["python", "app.py"]
