# City Chain Project

市町村のブロックチェーンのプログラム

## 概要
City Chain Projectは、市町村、地域、大陸、そしてグローバルレベルでのトランザクションを処理するためのブロックチェーンシステムです。このプロジェクトは、市町村ごとのチェーン、地域チェーン、大陸チェーン、そしてグローバルメインチェーンで構成されています。

## プロジェクト構成
- **Global Main Chain**: 大陸チェーンからのデータを集約するメインチェーン。
- **Continental Chain**: 市町村チェーンからのデータを集約し、メインチェーンに転送する中間チェーン。
- **Municipal Chain**: 市町村内でのトランザクションを処理するローカルチェーン。
- **DApps**: 分散型アプリケーションで、ユーザーがトランザクションを作成し、システムとインタラクションするためのインターフェースを提供します。
- **Mobile App**: DAppsのモバイルバージョンで、移動中のアクセスを提供します。
- **Lattice Signer Service**: トランザクションの署名と検証を行うサービス。

## フェデレーションモデルの流れ
フェデレーションモデルの流れ
City Chain Projectは、次のようなフェデレーションモデルに基づいてトランザクションを処理します。

1.DApps側からMunicipal Chainへの送信: ユーザーがDAppsを介して愛貨を送信すると、トランザクションが生成され、Municipal Chainに送信されます。

2.Municipal Chainでの処理: Municipal Chainは、トランザクションを受け取り、ローカルで処理するか、他の市町村に転送するかを判断します。送信者が受信を希望する場合、そのトランザクションはContinental Main Chainに転送されます。

3.Continental Main Chainでの保留管理: Continental Main Chainは、Municipal Chainから転送されたトランザクションを「send_pending」の状態で保留し、受信者からの確認を待ちます。受信者がトランザクションを受け入れると、「receive_pending」状態に遷移し、トランザクションはMunicipal Chainに返され、処理が進行します。

4.トランザクションの完了: Municipal Chainでトランザクションが承認され、愛貨が交換されます。その結果がContinental Main Chainに通知され、オフライン処理で全Continental Main ChainからのデータがGlobal Main Chainに集約されます。

5.Global Main Chainでのデータ保存: Global Main Chainは、すべてのContinental Main Chainから集められたデータを一元管理し、最終的な記録として保存します。

6.未受信トランザクションリストの管理: Continental Main Chainでは、未受信のトランザクションリストを6ヶ月間保持し、その期間中に受信されなかったトランザクションの愛貨額が時間とともに減額されます。

## セキュリティ統合
- **NTRU Encryption and Signatures**: トランザクションのセキュリティと署名の検証に使用。ラティスベースの暗号技術を用いることで、量子計算に対する耐性を持ち、システム全体のセキュリティを強化します。NTRUは、アルゴリズムの実装は複雑だが、ライブラリで簡単に使えます。多くの場合、量子耐性暗号のアルゴリズム自体は非常に高度で複雑な数学を伴います。
例えば、格子暗号やNTRUのような技術は、格子の性質や多項式演算に基づく非常に複雑な計算をしていますが、これらをゼロから実装するのは非常に手間がかかり、専門知識も必要です。しかし、暗号ライブラリ（例えば ntru ライブラリ）を使うことで、実際のコードは比較的シンプルになります。多くの暗号ライブラリは、複雑なアルゴリズムの内部を隠蔽し、開発者がシンプルなAPIで暗号化や署名などの機能を利用できるようにしているからです。例えば、次のような操作がコードでは簡単に実装できます：
　let ntru_param = NtruParam::default();
　let ntru_encrypt = NtruEncrypt::new(&ntru_param);
　let ciphertext = ntru_encrypt.encrypt(&plaintext);
　let decrypted_text = ntru_encrypt.decrypt(&ciphertext);
このように、量子耐性暗号を使うコード自体は複雑なものではなく、単にライブラリを使ってデータを暗号化・復号化する手続きに従うだけです。

- **Verifiable Credentials**: ユーザーのクレデンシャルの認証と整合性を確保するために使用。これにより、ユーザーの身元と行動の信頼性を保証し、システム全体の信頼性を向上させます。

## 環境セットアップ
1. リポジトリをクローンします：
    ```bash
    git clone https://github.com/your-repo/city_chain_project.git
    cd city_chain_project
    ```

2. Dockerを使用してプロジェクトをビルドし、起動します：
    ```bash
    docker-compose up --build
    ```

3. 各サービスが以下のポートで実行されます：
    - Global Main Chain: `http://localhost:8000`
    - Continental Chain: `http://localhost:8001`
    - Municipal Chain: `http://localhost:8002`
    - DApps: `http://localhost:5000`
    - Lattice Signer Service: `http://localhost:5001`
    - Mobile App: `http://localhost:3000`

## 使用方法
各サービスにはそれぞれのエンドポイントがあります。以下はMunicipal Chainの例です：

- **POST /transaction**
  - 新しいトランザクションを作成します。
  - リクエストボディ:
    ```json
    {
      "sender": "string",
      "receiver": "string",
      "amount": "float"
    }
    ```
  - レスポンス:
    ```json
    {
      "transaction_id": "string",
      "timestamp": "string"
    }
    ```

- **POST /add_block**
  - 新しいブロックを追加します。
  - リクエストボディ:
    ```json
    {
      "index": "integer",
      "timestamp": "string",
      "data": "string",
      "prev_hash": "string",
      "hash": "string"
    }
    ```
  - レスポンス:
    ```json
    {
      "status": "string"
    }
    ```

## 詳細設計

詳細な設計については [docs/architecture.md](./docs/architecture.md) を参照してください。

## データフロー

データフローの詳細については [docs/data_flow.md](./docs/data_flow.md) を参照してください。

## コンセンサスアルゴリズム

コンセンサスアルゴリズムの詳細については [docs/consensus_algorithms.md](./docs/consensus_algorithms.md) を参照してください。

## ポート設定

各コンポーネントのポート設定については [docs/port_configuration.md](./docs/port_configuration.md) を参照してください。

## 貢献方法
1. このリポジトリをフォークします。
2. 新しいブランチを作成します (`git checkout -b feature-branch`)。
3. 変更をコミットします (`git commit -am 'Add new feature'`)。
4. ブランチをプッシュします (`git push origin feature-branch`)。
5. プルリクエストを開きます。

## ライセンス
このプロジェクトはMITライセンスの下でライセンスされています。詳細については、`LICENSE` ファイルを参照してください。

## その他
このプロジェクトに関する質問や提案がある場合は、[Issues](https://github.com/SatoshiNakamoto1024/city_chain_project/issues) セクションを通じてお問い合わせください。
