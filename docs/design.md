# Design Document

## Overview
この文書は、city chainプロジェクトとその設計概要を提供します。

## Components
1. Main Chain
2. Continental Chain
3. Municipal Chain
4. DApps
5. Mobile App

## Security Integration
- **NTRU Encryption and Signatures**: トランザクションのセキュリティと署名の検証に使用。ラティスベースの暗号技術を用いることで、量子計算に対する耐性を持ち、システム全体のセキュリティを強化します。
- **Verifiable Credentials**: ユーザーのクレデンシャルの認証と整合性を確保するために使用。これにより、ユーザーの身元と行動の信頼性を保証し、システム全体の信頼性を向上させます。

## Data Flow
以下のコンポーネント間のデータフローを記述します。
- **Municipal Chain to Continental Chain**: 市町村チェーンから大陸チェーンへデータを送信。
- **Continental Chain to Main Chain**: 大陸チェーンからメインチェーンへデータを集約。
- **DApps and Mobile App**: ユーザーインターフェースとして機能し、ユーザーがトランザクションを作成、署名、および送信するためのインタラクションを提供。

## API Specifications
以下のAPIエンドポイントとその仕様をリストします。

### Municipal Chain API
- **POST /transaction**
  - **Description**: 新しいトランザクションを作成します。
  - **Request Body**: 
    ```json
    {
      "sender": "string",
      "receiver": "string",
      "amount": "float"
    }
    ```
  - **Response**: 
    ```json
    {
      "transaction_id": "string",
      "timestamp": "string"
    }
    ```

- **POST /add_block**
  - **Description**: 新しいブロックを追加します。
  - **Request Body**: 
    ```json
    {
      "index": "integer",
      "timestamp": "string",
      "data": "string",
      "prev_hash": "string",
      "hash": "string"
    }
    ```
  - **Response**: 
    ```json
    {
      "status": "string"
    }
    ```

### Continental Chain API
- **POST /transaction**
  - **Description**: 市町村チェーンからのトランザクションを受信し、処理します。
  - **Request Body**: 
    ```json
    {
      "sender": "string",
      "receiver": "string",
      "amount": "float"
    }
    ```
  - **Response**: 
    ```json
    {
      "transaction_id": "string",
      "timestamp": "string"
    }
    ```

- **POST /add_block**
  - **Description**: 市町村チェーンからのブロックを受信し、メインチェーンへ転送します。
  - **Request Body**: 
    ```json
    {
      "index": "integer",
      "timestamp": "string",
      "data": "string",
      "prev_hash": "string",
      "hash": "string"
    }
    ```
  - **Response**: 
    ```json
    {
      "status": "string"
    }
    ```

### Main Chain API
- **POST /transaction**
  - **Description**: 大陸チェーンからのトランザクションを受信し、処理します。
  - **Request Body**: 
    ```json
    {
      "sender": "string",
      "receiver": "string",
      "amount": "float"
    }
    ```
  - **Response**: 
    ```json
    {
      "transaction_id": "string",
      "timestamp": "string"
    }
    ```

- **POST /add_block**
  - **Description**: 大陸チェーンからのブロックを受信し、グローバルチェーンへ転送します。
  - **Request Body**: 
    ```json
    {
      "index": "integer",
      "timestamp": "string",
      "data": "string",
      "prev_hash": "string",
      "hash": "string"
    }
    ```
  - **Response**: 
    ```json
    {
      "status": "string"
    }
    ```

### Verifiable Credentials API
- **POST /issue_credential**
  - **Description**: ユーザーに対して新しいクレデンシャルを発行します。
  - **Request Body**: 
    ```json
    {
      "user_id": "string",
      "credential_data": "string"
    }
    ```
  - **Response**: 
    ```json
    {
      "credential_id": "string",
      "status": "string"
    }
    ```

- **POST /verify_credential**
  - **Description**: ユーザーのクレデンシャルを検証します。
  - **Request Body**: 
    ```json
    {
      "credential_id": "string",
      "user_id": "string"
    }
    ```
  - **Response**: 
    ```json
    {
      "status": "string"
    }
    ```

### NTRU Encryption and Signature API
- **POST /encrypt_data**
  - **Description**: データをNTRUアルゴリズムで暗号化します。
  - **Request Body**: 
    ```json
    {
      "data": "string"
    }
    ```
  - **Response**: 
    ```json
    {
      "encrypted_data": "string"
    }
    ```

- **POST /decrypt_data**
  - **Description**: NTRUアルゴリズムで暗号化されたデータを復号します。
  - **Request Body**: 
    ```json
    {
      "encrypted_data": "string"
    }
    ```
  - **Response**: 
    ```json
    {
      "data": "string"
    }
    ```

- **POST /sign_data**
  - **Description**: データをNTRUアルゴリズムで署名します。
  - **Request Body**: 
    ```json
    {
      "data": "string"
    }
    ```
  - **Response**: 
    ```json
    {
      "signature": "string"
    }
    ```

- **POST /verify_signature**
  - **Description**: NTRUアルゴリズムで署名されたデータを検証します。
  - **Request Body**: 
    ```json
    {
      "data": "string",
      "signature": "string"
    }
    ```
  - **Response**: 
    ```json
    {
      "valid": "boolean"
    }
    ```

## Consensus Algorithms
システムの整合性とセキュリティを確保するために、さまざまなコンセンサスアルゴリズムを使用します。

### Delegated Proof of Stake (DPoS)
#### Struct Definition
```rust
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
