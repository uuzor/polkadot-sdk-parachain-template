import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import './globals.css';
import Navbar from '@/components/Navbar';

const inter = Inter({ subsets: ['latin'] });

export const metadata: Metadata = {
  title: 'BattleChain - Gaming Prediction Market',
  description: 'Decentralized NFT battles with prediction markets and verified game results',
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={inter.className}>
        <div className="min-h-screen flex flex-col">
          <Navbar />
          <main className="flex-1">
            {children}
          </main>
          <footer className="border-t border-dark-border py-8">
            <div className="container mx-auto px-4 text-center text-gray-400">
              <p>© 2024 BattleChain. Built on Polkadot SDK.</p>
            </div>
          </footer>
        </div>
      </body>
    </html>
  );
}
