import React, { useState } from 'react';
import { Button, TextInput, View, Text } from 'react-native';
import axios from 'axios';
// 仮のNTRUライブラリ
import { NtruEncrypt, NtruDecrypt, NtruSign, NtruVerify } from 'ntru-crypto';

export default function App() {
  const [action, setAction] = useState('');
  const [response, setResponse] = useState('');
  const [publicKey, setPublicKey] = useState(''); // 公開鍵を保持する状態
  const [privateKey, setPrivateKey] = useState(''); // 秘密鍵を保持する状態

  const sendAction = async () => {
    try {
      // Verifiable Credentialを取得
      const credentialResponse = await axios.post('http://localhost:5000/get_credential', {
        action,
      });

      const credential = credentialResponse.data;

      // ActionをNTRUで暗号化
      const encryptedAction = NtruEncrypt(action, publicKey);
      // Actionに署名
      const signature = NtruSign(action, privateKey);

      // Encrypted Action, Credential, Signatureを送信
      const result = await axios.post('http://localhost:5000/add_action', {
        action: encryptedAction,
        credential: credential,
        signature: signature,
      });

      setResponse(result.data);
    } catch (error) {
      console.error(error);
    }
  };

  const generateKeys = async () => {
    try {
      // 鍵ペア生成エンドポイントにリクエスト
      const keyResponse = await axios.post('http://localhost:5000/generate_keys');
      const { publicKey, privateKey } = keyResponse.data;
      setPublicKey(publicKey);
      setPrivateKey(privateKey);
    } catch (error) {
      console.error(error);
    }
  };

  return (
    <View style={{ padding: 20 }}>
      <Text>Enter action:</Text>
      <TextInput
        value={action}
        onChangeText={setAction}
        style={{ height: 40, borderColor: 'gray', borderWidth: 1, marginBottom: 20 }}
      />
      <Button title="Generate Keys" onPress={generateKeys} />
      <Button title="Send Action" onPress={sendAction} />
      <Text>Response: {response}</Text>
    </View>
  );
}
