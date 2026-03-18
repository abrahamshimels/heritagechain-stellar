# HeritageChain 🔷


## 🌍 Overview
HeritageChain is a blockchain-powered platform for purchasing verified digital representations of **Ethiopian cultural heritage sites**. It automatically distributes revenue among stakeholders (Treasury, Preservation Fund, and Artists) using Soroban smart contracts on the Stellar network.

## 🎯 Problem
Traditional tourism revenue distribution lacks transparency, automation, and a direct link to actual cultural preservation efforts.

## 💡 Solution
- **Digital Ownership**: Collectors can "own" a piece of history.
- **Automated Revenue Splitting**: 70% to Treasury, 20% to Preservation, 10% to Artists.
- **Transparency**: Every transaction is verifiable on the Stellar blockchain.

## ⚙️ Core Features
- 🔐 **Freighter Wallet** integration.
- 🏛️ **Heritage Gallery**: Browse unique collectibles.
- 💳 **One-Click Purchase**: Secure and instant.
- 📜 **On-Chain Tracking**: Proof of ownership stored permanently.

## 🏗️ Architecture
`Frontend (React/TS)` → `Blockchain Service Layer` → `Freighter Wallet` → `Soroban Contracts (Rust)` → `Stellar Network`

## 🧱 Tech Stack
- **Frontend**: React, Vite, TypeScript, Vanilla CSS.
- **Blockchain**: Soroban SDK, Stellar SDK.
- **Wallet**: Freighter.

## 🚀 Getting Started
1. **Clone Repo**
   ```bash
   git clone https://github.com/abrahamshimels/heritagechain-stellar.git
   cd heritagechain
   ```
2. **Setup Frontend**
   ```bash
   cd apps/web
   npm install
   npm run dev
   ```
3. **Build Contracts**
   ```bash
   cd contracts/heritagechain
   cargo build --target wasm32-unknown-unknown --release
   ```

## 👥 Team: Rustfarian
Built during the **Stellar & Soroban Smart Contract Bootcamp Hackathon**.

1. **Abraham Shimels (Leader)** - [shimelsabraham123@gmail.com](mailto:shimelsabraham123@gmail.com)
2. **Samuel Birhanu** - [samwoker112@gmail.com](mailto:samwoker112@gmail.com)
3. **Abdurazak Mohammed** - [abdulkem5472@gmail.com](mailto:abdulkem5472@gmail.com)
4. **Hermela Ijigu** - [Hermelaejigu4@gmail.com](mailto:Hermelaejigu4@gmail.com)
5. **Natinael Abiyu** - [Natiabiyu22@gmail.com](mailto:Natiabiyu22@gmail.com)
