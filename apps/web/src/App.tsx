import React, { useState } from 'react';
import { WalletConnection } from './features/wallet/WalletConnection';
import { CollectibleGallery } from './features/collectibles/CollectibleGallery';
import { MyCollection } from './features/user/MyCollection';
import { checkWalletConnection } from './services/stellar/wallet';
import './index.css';

function App() {
  const [publicKey, setPublicKey] = useState<string | null>(null);

  React.useEffect(() => {
    checkWalletConnection().then(setPublicKey);
  }, []);

  const handlePurchaseSuccess = () => {
    checkWalletConnection().then(setPublicKey); 
  };

  return (
    <div className="container">
      <header className="header">
        <h1>HeritageChain</h1>
        <WalletConnection />
      </header>
      
      <main className="main-content">
        <section className="intro">
          <p>Purchase authenticated digital collectibles of Ethiopian heritage sites.</p>
          <p>Every purchase automatically splits: <b>70% to Treasury, 20% to Site Fund, 10% to Artist</b>.</p>
        </section>

        <CollectibleGallery publicKey={publicKey} onPurchaseSuccess={handlePurchaseSuccess} />
        <MyCollection publicKey={publicKey} />
      </main>
    </div>
  );
}

export default App;
