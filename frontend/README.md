# BattleChain Frontend

A modern, responsive web application for interacting with the BattleChain gaming prediction market platform built on Polkadot SDK.

## 🎨 Design Features

- **Dark Theme**: Immersive gaming aesthetic with dark backgrounds
- **Color Palette**:
  - Purple/Violet (#a855f7, #9333ea) - Primary theme
  - Cyan/Blue (#06b6d4, #22d3ee) - Accent colors
  - Dark backgrounds (#0a0a0f, #13131a)
- **Modern UI Elements**:
  - Glassmorphism effects
  - Gradient overlays
  - Smooth animations
  - Hover effects with glowing shadows
  - Responsive card-based layouts

## 🚀 Tech Stack

- **Framework**: Next.js 14 (App Router)
- **Styling**: Tailwind CSS
- **Blockchain**: Polkadot.js API
- **State Management**: Zustand
- **Icons**: React Icons
- **Language**: TypeScript

## 📦 Installation

1. Navigate to the frontend directory:

```bash
cd frontend
```

2. Install dependencies:

```bash
npm install
# or
yarn install
# or
pnpm install
```

## 🔧 Configuration

Create a `.env.local` file in the frontend directory:

```env
NEXT_PUBLIC_WS_PROVIDER=ws://127.0.0.1:9944
```

Adjust the WebSocket provider URL to match your parachain node.

## 🏃 Running the Application

### Development Mode

```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser.

### Production Build

```bash
npm run build
npm start
```

## 📱 Features

### ⚔️ Battles Page

- **Character Creation**: Choose from 5 unique character classes
  - Warrior: Balanced stats, high HP
  - Assassin: High crit chance, burst damage
  - Mage: Moderate damage, good crit
  - Tank: Maximum HP and defense
  - Trickster: Balanced with high dodge

- **Battle Management**: Create and accept battle offers (Coming Soon)
- **Battle Execution**: Watch rounds-based combat (Coming Soon)

### 🎲 Markets Page

- **Market Creation**: Create prediction markets for battles (Coming Soon)
- **Place Predictions**: Bet on battle outcomes (Coming Soon)
- **AMM Pools**: View liquidity pools for each outcome (Coming Soon)
- **Claim Winnings**: Collect rewards from successful predictions (Coming Soon)

### 🔮 Oracle Page

- **Developer Registration**: Register with staking (Coming Soon)
- **Result Submission**: Submit verified game results (Coming Soon)
- **Dispute Management**: Challenge or defend results (Coming Soon)
- **Revenue Tracking**: View earnings from data queries (Coming Soon)

## 🎯 Project Structure

```
frontend/
├── src/
│   ├── app/                 # Next.js App Router pages
│   │   ├── battles/         # Battle page
│   │   ├── markets/         # Prediction markets page
│   │   ├── oracle/          # Game oracle page
│   │   ├── layout.tsx       # Root layout
│   │   ├── page.tsx         # Homepage
│   │   └── globals.css      # Global styles
│   ├── components/          # React components
│   │   └── Navbar.tsx       # Navigation with wallet connection
│   ├── lib/                 # Utility functions
│   │   └── polkadot.ts      # Polkadot.js API integration
│   ├── stores/              # Zustand state management
│   │   └── walletStore.ts   # Wallet connection state
│   ├── hooks/               # Custom React hooks (planned)
│   └── types/               # TypeScript type definitions (planned)
├── public/                  # Static assets
├── next.config.js           # Next.js configuration
├── tailwind.config.js       # Tailwind CSS configuration
├── tsconfig.json            # TypeScript configuration
└── package.json             # Dependencies
```

## 🔗 Wallet Connection

The application uses Polkadot.js extension for wallet management:

1. Install [Polkadot.js extension](https://polkadot.js.org/extension/) for your browser
2. Create or import an account
3. Click "Connect Wallet" in the navbar
4. Select your account from the dropdown

## 🎨 UI Components

### Tailwind Classes

Custom utility classes available:

- `.card` - Standard card styling
- `.card-hover` - Card with hover effects
- `.btn-primary` - Primary gradient button
- `.btn-secondary` - Secondary gradient button
- `.btn-outline` - Outlined button
- `.input` - Styled input field
- `.stat-card` - Statistics display card

### Character Classes

Character-specific gradient backgrounds:

- `.character-class-warrior` - Red gradient
- `.character-class-assassin` - Purple gradient
- `.character-class-mage` - Blue gradient
- `.character-class-tank` - Green gradient
- `.character-class-trickster` - Yellow gradient

## 🧪 Development Tips

### Adding New Pages

1. Create a new directory in `src/app/`
2. Add `page.tsx` for the route
3. Update navigation in `Navbar.tsx` if needed

### Working with Polkadot.js

```typescript
import { getApi, getInjector } from '@/lib/polkadot';

// Get API instance
const api = await getApi();

// Sign and send transaction
const injector = await getInjector(address);
await api.tx.palletName.extrinsicName(params)
  .signAndSend(address, { signer: injector.signer }, callback);
```

### State Management

```typescript
import { useWalletStore } from '@/stores/walletStore';

// In component
const { api, selectedAccount, isConnected } = useWalletStore();
```

## 🐛 Troubleshooting

### Wallet Not Connecting

- Ensure Polkadot.js extension is installed
- Check that the extension has permission for the site
- Refresh the page and try again

### Transaction Failures

- Verify your account has sufficient balance
- Check the parachain node is running
- Ensure the WebSocket connection is active

### Build Errors

- Clear Next.js cache: `rm -rf .next`
- Delete node_modules and reinstall: `rm -rf node_modules && npm install`

## 📚 Resources

- [Next.js Documentation](https://nextjs.org/docs)
- [Tailwind CSS](https://tailwindcss.com/docs)
- [Polkadot.js API](https://polkadot.js.org/docs/api)
- [Polkadot SDK](https://paritytech.github.io/polkadot-sdk/master/)

## 🚧 Roadmap

- [x] Homepage with feature showcase
- [x] Wallet connection
- [x] Character creation UI
- [ ] Battle list and details
- [ ] Battle offer creation
- [ ] Battle execution visualization
- [ ] Prediction market creation
- [ ] Place predictions interface
- [ ] Market resolution and claiming
- [ ] Developer registration
- [ ] Result submission form
- [ ] Dispute management
- [ ] Real-time updates with subscriptions
- [ ] Mobile responsiveness improvements
- [ ] Dark/Light theme toggle

## 📄 License

MIT-0

---

Built with ❤️ using Next.js, Tailwind CSS, and Polkadot SDK
