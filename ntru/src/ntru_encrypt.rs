pub struct NtruEncrypt;

impl NtruEncrypt {
    pub fn new() -> Self {
        NtruEncrypt
    }

    pub fn encrypt(&self, message: &str, public_key: &[u8]) -> Vec<u8> {
        // 簡単な暗号化処理の例
        message.bytes().zip(public_key.iter().cycle()).map(|(m, k)| m ^ k).collect()
    }

    pub fn decrypt(&self, encrypted_message: &[u8], private_key: &[u8]) -> String {
        // 簡単な復号化処理の例
        encrypted_message.iter().zip(private_key.iter().cycle()).map(|(e, k)| (e ^ k) as char).collect()
    }
}
