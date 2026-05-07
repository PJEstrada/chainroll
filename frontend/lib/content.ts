import {
  BadgeCheck,
  Banknote,
  Clock3,
  FileCheck2,
  ShieldCheck,
  WalletCards,
} from "lucide-react";

export const navLinks = [
  { label: "Features", href: "#capabilities" },
  { label: "How It Works", href: "#how-it-works" },
  { label: "Pricing", href: "#pricing" },
  { label: "Support", href: "#support" },
];

export const trustedBy = [
  "Tempo Testnet",
  "Remote Crew",
  "ShiftWorks",
  "USDC Ops",
  "Mercado Teams",
  "Boto Pilot",
];

export const features = [
  {
    icon: BadgeCheck,
    title: "Tenant-scoped worker roster",
    description:
      "Create, update, count, and verify employees with clean tenant boundaries built into every request.",
  },
  {
    icon: WalletCards,
    title: "Wallet-ready payroll records",
    description:
      "Store checksummed EVM wallet addresses on employees so payout readiness is visible before payroll day.",
  },
  {
    icon: Clock3,
    title: "Compensation profiles",
    description:
      "Model token amounts, weekly, biweekly, monthly, or custom cadence, and validity windows per worker.",
  },
  {
    icon: Banknote,
    title: "Tempo treasury setup",
    description:
      "Configure default treasury accounts with token settings, custody provider, and execution control mode.",
  },
  {
    icon: FileCheck2,
    title: "Audit-backed changes",
    description:
      "Track treasury and compensation mutations with actor-aware audit events for operational review.",
  },
  {
    icon: ShieldCheck,
    title: "Preview before execution",
    description:
      "Surface missing wallets, missing compensation, token mismatches, and signature-required blockers before money moves.",
  },
];

export const howItWorksSteps = [
  {
    number: "01",
    title: "Add workers",
    description:
      "Capture roster details, identifiers, divisions, locale, attributes, and wallet addresses under one tenant.",
  },
  {
    number: "02",
    title: "Configure pay",
    description:
      "Attach active compensation profiles and match token symbols to default Tempo treasury accounts.",
  },
  {
    number: "03",
    title: "Preview and execute",
    description:
      "Review payable totals, resolve blockers, then submit ready payout attempts when the backend endpoint arrives.",
  },
];

export const footerColumns = [
  {
    title: "Product",
    links: [
      "Payroll preview",
      "Treasury accounts",
      "Worker wallets",
      "Audit events",
    ],
  },
  {
    title: "Company",
    links: ["About", "Careers", "Capstone", "Contact"],
  },
  {
    title: "Resources",
    links: ["Docs", "API status", "Support", "Security"],
  },
  {
    title: "Legal",
    links: ["Privacy", "Terms", "Compliance", "Disclosures"],
  },
];
