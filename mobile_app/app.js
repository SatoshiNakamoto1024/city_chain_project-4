import React, { useState } from 'react';
import { Button, TextInput, View, Text } from 'react-native';
import axios from 'axios';

export default function App() {
  const [action, setAction] = useState('');
  const [response, setResponse] = useState('');

  const sendAction = async () => {
    try {
      const result = await axios.post('http://localhost:5000/add_action', {
        action,
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
