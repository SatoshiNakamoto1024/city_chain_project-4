import React, { useState } from 'react';
import { Button, TextInput, View, Text } from 'react-native';
import axios from 'axios';
import { NtruEncrypt, NtruDecrypt, NtruSign, NtruVerify } from 'ntru-crypto';  // 仮のNTRUライブラリ

export default function App() {
  const [action, setAction] = useState('');
  const [response, setResponse] = useState('');
  const [publicKey, setPublicKey] = useState('');  // 公開鍵を保持する状態
  const [privateKey, setPrivateKey] = useState('');  // 秘密鍵を保持する状態

  const sendAction = async () => {
    try {
      // Verifiable Credentialを取得
      const credential = await axios.post('http://localhost:5000/get_credential', {
        action,
      });

      const encryptedAction = NtruEncrypt(action, publicKey);
      const signature = NtruSign(action, privateKey);

      const result = await axios.post('http://localhost:5000/add_action', {
        action: encryptedAction,
        credential: credential.data,
        signature: signature,
      });

      setResponse(result.data);
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
      <Button title="Send Action" onPress={sendAction} />
      <Text>Response: {response}</Text>
    </View>
  );
}
