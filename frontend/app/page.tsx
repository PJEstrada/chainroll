import { CtaBanner } from "@/components/cta-banner";
import { Features } from "@/components/features";
import { Footer } from "@/components/footer";
import { Hero } from "@/components/hero";
import { HowItWorks } from "@/components/how-it-works";
import { Nav } from "@/components/nav";
import { TransactionStatusDemo } from "@/components/transaction-status-demo";
import { TrustedBy } from "@/components/trusted-by";

export default function Home() {
  return (
    <main className="min-h-screen bg-surface-page">
      <Nav />
      <Hero />
      <TrustedBy />
      <Features />
      <HowItWorks />
      <TransactionStatusDemo />
      <CtaBanner />
      <Footer />
    </main>
  );
}
