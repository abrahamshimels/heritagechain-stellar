import React, { useEffect, useState } from 'react';
import { checkWalletConnection, connectWallet } from '../../services/stellar/wallet';

export const WalletConnection: React.FC = () => {
  const [publicKey, setPublicKey] = useState<string | null>(null);

  useEffect(() => {
    checkWalletConnection().then(setPublicKey);
  }, []);

  const handleConnect = async () => {
    const key = await connectWallet();
    setPublicKey(key);
  };

  return (
    <div className="wallet-connection">
      {publicKey ? (
        <span className="address">
          Connected: {publicKey.substring(0, 5)}...{publicKey.slice(-4)}
        </span>
      ) : (
        <button className="btn-primary" onClick={handleConnect}>
          Connect Freighter
        </button>
      )}
    </div>
  );
};
