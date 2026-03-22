import { isAllowed, setAllowed, requestAccess } from '@stellar/freighter-api';

export const checkWalletConnection = async (): Promise<string | null> => {
  if (await isAllowed()) {
    const response = await requestAccess();
    if (response?.address) {
      return response.address;
    }
  }
  return null;
};

export const connectWallet = async (): Promise<string | null> => {
  try {
    await setAllowed();
    return await checkWalletConnection();
  } catch (e) {
    console.error("Wallet connection failed", e);
    return null;
  }
};
