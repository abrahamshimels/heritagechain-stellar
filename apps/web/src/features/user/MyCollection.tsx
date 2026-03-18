import React, { useEffect, useState } from 'react';
import { getCollectibles, type Collectible } from '../../services/stellar/contract';

interface Props {
  publicKey: string | null;
}

export const MyCollection: React.FC<Props> = ({ publicKey }) => {
  const [myItems, setMyItems] = useState<Collectible[]>([]);

  useEffect(() => {
    if (publicKey) {
      // Mock fetching user's items
      // For demo, we just grab one from the list as "owned"
      getCollectibles().then(items => setMyItems(items.slice(0, 1))); 
    }
  }, [publicKey]);

  if (!publicKey) return null;

  return (
    <div className="my-collection">
      <h2>My Collection</h2>
      <div className="grid">
        {myItems.map(item => (
          <div className="card owned" key={item.id}>
            <h3>{item.name}</h3>
            <p>✅ Owned by you</p>
          </div>
        ))}
      </div>
    </div>
  );
};
