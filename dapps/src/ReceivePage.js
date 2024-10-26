import React, { useEffect, useState } from 'react';
import axios from 'axios';

function ReceivePage() {
  const [pendingTransactions, setPendingTransactions] = useState([]);

  useEffect(() => {
    // Municipal Chainから保留トランザクションを取得
    axios.get('http://localhost:8000/pending_transactions') // Municipal ChainのAPIエンドポイントに合わせてURLを変更
      .then(response => {
        setPendingTransactions(response.data);
      })
      .catch(error => {
        console.error('Error fetching pending transactions:', error);
      });
  }, []);

  const handleReceive = (transactionId) => {
    // トランザクションを受信するリクエストを送信
    axios.post('http://localhost:8000/receive_transaction', { transactionId }) // Municipal Chainにトランザクションの受信を通知
      .then(response => {
        console.log('Transaction received:', response.data);
        // トランザクション受信後、リストから削除
        setPendingTransactions(pendingTransactions.filter(tx => tx.transactionId !== transactionId));
      })
      .catch(error => {
        console.error('Error receiving transaction:', error);
      });
  };

  return (
    <div className="ReceivePage">
      <h1>保留中のトランザクション</h1>
      <ul>
        {pendingTransactions.map(tx => (
          <li key={tx.transactionId}>
            <span>送信者: {tx.sender} | 金額: {tx.amount}</span>
            <button onClick={() => handleReceive(tx.transactionId)}>受信</button>
          </li>
        ))}
      </ul>
    </div>
  );
}

export default ReceivePage;
