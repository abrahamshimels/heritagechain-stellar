import React, { useState } from 'react';
import { purchaseCollectible, type Collectible } from '../../services/stellar/contract';

interface Props {
  collectible: Collectible;
  publicKey: string | null;
  onSuccess: () => void;
}

export const PurchaseButton: React.FC<Props> = ({ collectible, publicKey, onSuccess }) => {
  const [loading, setLoading] = useState(false);

  const handlePurchase = async () => {
    if (!publicKey) return alert("Please connect wallet first");
    
    setLoading(true);
    try {
      await purchaseCollectible(publicKey, collectible.id);
      alert("Purchase successful! 70% to treasury, 20% to site, 10% to artist.");
      onSuccess();
    } catch (e) {
      console.error(e);
      alert("Purchase failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <button 
      className="btn-primary" 
      onClick={handlePurchase} 
      disabled={loading || !publicKey}
    >
      {loading ? 'Processing...' : `Buy for ${collectible.price} XLM`}
    </button>
  );
};
