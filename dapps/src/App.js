import React, { useState } from 'react';
import { BrowserRouter as Router, Route, Routes, Link } from 'react-router-dom';
import './App.css';
import axios from 'axios';

function SendPage() {
  const [formData, setFormData] = useState({
    sender: '',
    receiver: '',
    amount: '',
    subject: '',
    actionLevel: '',
    dimension: '',
    fluctuation: '',
    organismName: '',
    municipality: '',
    details: '',
    goodsOrMoney: ''
  });

  const users = [
    { name: 'Aさん', municipality: '加賀市' },
    { name: 'Bさん', municipality: '小松市' },
    { name: 'Cさん', municipality: '能美市' },
    { name: 'Dさん', municipality: '白山市' },
    { name: 'Eさん', municipality: '加賀市' },
    { name: 'Fさん', municipality: '小松市' },
    { name: 'Gさん', municipality: '能美市' },
    { name: 'Hさん', municipality: '能美市' },
    { name: 'Iさん', municipality: '加賀市' },
    { name: 'Kさん', municipality: '小松市' },
  ];

  const handleChange = (e) => {
    const { name, value } = e.target;
    setFormData({
      ...formData,
      [name]: value,
    });
  };

  const handleSubmit = (e) => {
    e.preventDefault();

    // Flaskサーバーにデータを送信
    axios.post('http://localhost:5000/send', formData)
        .then(response => {
            console.log('Success:', response);
            // フォームをクリア
            setFormData({
                sender: '',
                receiver: '',
                amount: '',
                subject: '',
                actionLevel: '',
                dimension: '',
                fluctuation: '',
                organismName: '',
                municipality: '',
                details: '',
                goodsOrMoney: ''
            });
            // メッセージを表示
            alert('データを送信しました！');
        })
        .catch(error => {
            console.error('Error:', error);
            // エラーメッセージを表示
            alert('データの送信に失敗しました。');
        });
};

  return (
    <div className="SendPage">
      <h1>愛貨を送る</h1>
      <form onSubmit={handleSubmit}>
        <label>送信者:</label>
        <select name="sender" value={formData.sender} onChange={handleChange} required>
          <option value="">選択してください</option>
          {users.map(user => (
            <option key={user.name} value={user.name}>{user.name} ({user.municipality})</option>
          ))}
        </select>
        <br />

        <label>受信者:</label>
        <select name="receiver" value={formData.receiver} onChange={handleChange} required>
          <option value="">選択してください</option>
          {users.map(user => (
            <option key={user.name} value={user.name}>{user.name} ({user.municipality})</option>
          ))}
        </select>
        <br />

        <label>愛貨額:</label>
        <input type="number" name="amount" value={formData.amount} onChange={handleChange} required />
        <br />

        <label>科目:</label>
        <input type="text" name="subject" value={formData.subject} onChange={handleChange} />
        <br />

        <label>行動レベル:</label>
        <input type="text" name="actionLevel" value={formData.actionLevel} onChange={handleChange} />
        <br />

        <label>次元:</label>
        <input type="text" name="dimension" value={formData.dimension} onChange={handleChange} />
        <br />

        <label>ゆらぎ:</label>
        <input type="text" name="fluctuation" value={formData.fluctuation} onChange={handleChange} />
        <br />

        <label>生命体名:</label>
        <input type="text" name="organismName" value={formData.organismName} onChange={handleChange} />
        <br />

        <label>所属市町村:</label>
        <input type="text" name="municipality" value={formData.municipality} onChange={handleChange} />
        <br />

        <label>詳細内容:</label>
        <input type="text" name="details" value={formData.details} onChange={handleChange} />
        <br />

        <label>付随してモノやお金を渡す:</label>
        <input type="text" name="goodsOrMoney" value={formData.goodsOrMoney} onChange={handleChange} />
        <br />

        <button type="submit">愛貨を送信</button>
      </form>
      <Link to="/receive">受信ページへ</Link>
    </div>
  );
}

function ReceivePage() {
  const [pendingTransactions, setPendingTransactions] = useState([]);

  const fetchPendingTransactions = () => {
    axios.get('/pending_transactions')
      .then(response => {
        setPendingTransactions(response.data);
      })
      .catch(error => {
        console.error('Error fetching pending transactions:', error);
      });
  };

  const handleReceive = (transactionId) => {
    axios.post('/receive_transaction', { transaction_id: transactionId })
      .then(response => {
        console.log('Received:', response);
        fetchPendingTransactions();  // リストのリフレッシュ
      })
      .catch(error => {
        console.error('Error receiving transaction:', error);
      });
  };

  return (
    <div className="ReceivePage">
      <h1>愛貨を受信する</h1>
      <button onClick={fetchPendingTransactions}>保留トランザクションを取得</button>
      <ul>
        {pendingTransactions.map((tx) => (
          <li key={tx.transaction_id}>
            <p>送信者: {tx.sender}</p>
            <p>受信者: {tx.receiver}</p>
            <p>金額: {tx.amount}</p>
            <button onClick={() => handleReceive(tx.transaction_id)}>受信</button>
          </li>
        ))}
      </ul>
      <Link to="/">送信ページへ戻る</Link>
    </div>
  );
}

function App() {
  return (
    <Router>
      <Routes>
        <Route path="/receive" element={<ReceivePage />} />
        <Route path="/" element={<SendPage />} />
      </Routes>
    </Router>
  );
}

export default App;

