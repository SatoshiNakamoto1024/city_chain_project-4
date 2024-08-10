pub struct NtruSign;

impl NtruSign {
    pub fn new() -> Self {
        NtruSign
    }

    pub fn sign(&self, message: &str, private_key: &[u8]) -> Vec<u8> {
        // 簡単な署名処理の例
        message.bytes().zip(private_key.iter().cycle()).map(|(m, k)| m ^ k).collect()
    }

    pub fn verify(&self, message: &str, signature: &[u8], public_key: &[u8]) -> bool {
        // 簡単な署名検証処理の例
        let computed_signature: Vec<u8> = message.bytes().zip(public_key.iter().cycle()).map(|(m, k)| m ^ k).collect();
        computed_signature == signature
    }
}
