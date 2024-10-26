import axios from 'axios';

export const sendTransaction = async (transactionData) => {
    try {
        const response = await axios.post('/create_transaction', transactionData);
        return response.data;
    } catch (error) {
        console.error("Transaction failed:", error);
        return { error: "Transaction failed" };
    }
};
