# フェデレーションモデルの詳細設計

## システム全体の概要

このドキュメントでは、フェデレーションモデルにおけるデータフローと各コンポーネントの役割について詳細に説明します。このシステムは、送信用および受信用のインターフェース、複数のブロックチェーンチェーン（市町村チェーン、大陸チェーン、グローバルメインチェーン）、MongoDBによるデータ保存、DAppsおよびモバイルアプリを含む分散型アプリケーションを組み合わせて構築されています。

## コンポーネントの詳細

### 1. Global Main Chain（グローバルメインチェーン）

- **役割**: 各大陸チェーンから集約されたデータを統合・管理します。システム全体のデータの一元管理を行い、信頼性と整合性を確保します。
- **機能**:
  - 大陸チェーンからのブロックを受け取り、承認・統合。
  - データの整合性チェックと保護。
  - 保留リストの管理と共有。

### 2. Continental Chain（大陸チェーン）

- **役割**: 市町村チェーンから収集されたデータをグローバルメインチェーンに転送します。大陸レベルでのデータ管理と処理を担当します。
- **機能**:
  - 市町村チェーンからトランザクションデータを受け取る。
  - データの整合性と承認プロセスの実施。
  - グローバルメインチェーンへのブロック転送。

### 3. Municipal Chain（市町村チェーン）

- **役割**: 各市町村内でのトランザクションを処理します。ローカルなデータ管理とトランザクションの承認を行います。
- **機能**:
  - トランザクションの受信と保存（send_pendingおよびreceive_pending）。
  - トランザクションの承認とブロック生成。
  - トランザクションのMongoDBへの保存。

### 4. DApps（分散型アプリケーション）

- **役割**: ユーザーインターフェースを提供し、ユーザーがトランザクションを送信・受信できるようにします。
- **機能**:
  - トランザクションの作成・送信。
  - トランザクションのステータス確認。
  - データの視覚化と操作。

### 5. Mobile App（モバイルアプリ）

- **役割**: DAppsのモバイルバージョンで、外出先からのアクセスと操作を可能にします。
- **機能**:
  - モバイルデバイスからのトランザクション送信・受信。
  - プッシュ通知によるリアルタイム更新。
  - モバイル特有のユーザーインターフェースの提供。

### 6. MongoDB

- **役割**: トランザクションデータやブロックデータを永続的に保存します。データの検索や分析にも利用されます。
- **機能**:
  - トランザクションのステータス管理（pending、completeなど）。
  - ブロックデータの保存と管理。
  - データのバックアップとリストア。

## コンセンサスアルゴリズム

システムのブロックチェーンネットワークの整合性とセキュリティを確保するために、複数のコンセンサスアルゴリズムが採用されています。

### Delegated Proof of Stake (DPoS)

#### 構造定義

rust
struct DPoS {
    municipalities: Vec<String>,
    approved_representative: Option<String>,
}

impl DPoS {
    fn new(municipalities: Vec<String>) -> Self {
        Self {
            municipalities,
            approved_representative: None,
        }
    }

    fn elect_representative(&mut self) -> String {
        let representative = self.municipalities.choose(&mut rand::thread_rng()).unwrap().clone();
        self.approved_representative = Some(representative.clone());
        representative
    }

    fn approve_transaction(&self, transaction: &mut Transaction) -> Result<&str, &str> {
        if let Some(representative) = &self.approved_representative {
            transaction.signature = format!("approved_by_{}", representative);
            Ok("Transaction approved")
        } else {
            Err("No representative elected")
        }
    }
}