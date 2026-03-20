import React, { useEffect, useState } from 'react';
import { getCollectibles, type Collectible } from '../../services/stellar/contract';
import { PurchaseButton } from '../purchase/PurchaseButton';

interface Props {
  publicKey: string | null;
  onPurchaseSuccess: () => void;
}

export const CollectibleGallery: React.FC<Props> = ({ publicKey, onPurchaseSuccess }) => {
  const [collectibles, setCollectibles] = useState<Collectible[]>([]);

  useEffect(() => {
    getCollectibles().then(setCollectibles);
  }, []);

  return (
    <div className="gallery">
      <h2>Heritage Collectibles</h2>
      <div className="grid">
        {collectibles.map((item) => (
          <div className="card" key={item.id}>
            <h3>{item.name}</h3>
            <p className="site">📍 {item.site}</p>
            <p className="price">{item.price} XLM</p>
            <PurchaseButton 
              collectible={item} 
              publicKey={publicKey} 
              onSuccess={onPurchaseSuccess} 
            />
          </div>
        ))}
      </div>
    </div>
  );
};
