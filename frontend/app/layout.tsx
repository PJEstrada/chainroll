import type { Metadata, Viewport } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import type * as React from "react";

import { ThemeProvider } from "@/components/theme-provider";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
  display: "swap",
  preload: true,
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
  display: "swap",
  preload: true,
});

export const metadata: Metadata = {
  title: "Boto | Cross-border payroll for workers",
  description:
    "Boto helps teams preview, review, and execute stablecoin payroll for cross-border workers.",
  applicationName: "Boto",
  openGraph: {
    title: "Boto | Cross-border payroll for workers",
    description:
      "Preview payroll readiness, resolve blockers, and move stablecoin payments across borders.",
    siteName: "Boto",
    type: "website",
  },
};

export const viewport: Viewport = {
  themeColor: [
    {
      media: "(prefers-color-scheme: dark)",
      color: "var(--color-surface-page)",
    },
    {
      media: "(prefers-color-scheme: light)",
      color: "var(--color-surface-page)",
    },
  ],
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      className={`${geistSans.variable} ${geistMono.variable}`}
      data-theme="dark"
      lang="en"
      suppressHydrationWarning
    >
      <body>
        <ThemeProvider>{children}</ThemeProvider>
      </body>
    </html>
  );
}
