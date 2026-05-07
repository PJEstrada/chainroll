import { ArrowRight, PlayCircle } from "lucide-react";

import { FadeUp } from "@/components/fade-up";
import { Button } from "@/components/ui/button";

export function Hero() {
  return (
    <section className="relative overflow-hidden" id="top">
      <div className="pointer-events-none absolute left-1/2 top-24 h-64 w-64 -translate-x-1/2 rounded-full bg-brand-blush opacity-70 blur-3xl lg:h-96 lg:w-96" />
      <div className="mx-auto flex min-h-[calc(100svh-4rem)] max-w-6xl flex-col justify-center px-6 py-24 lg:px-8">
        <FadeUp className="max-w-4xl" mode="mount">
          <p className="mb-5 text-sm font-semibold uppercase tracking-normal text-ink-muted">
            Cross-border payroll, built for stablecoin rails
          </p>
          <h1 className="text-6xl font-bold leading-[1.05] tracking-tight text-ink-primary md:text-7xl">
            Money that moves. Workers who don&apos;t wait.
          </h1>
          <p className="mt-6 max-w-2xl text-lg leading-relaxed text-ink-secondary">
            Boto helps teams preview payroll readiness, resolve wallet and
            treasury blockers, and prepare instant cross-border payments for
            workers.
          </p>
          <div className="mt-9 flex flex-col gap-3 sm:flex-row">
            <Button asChild size="lg">
              <a href="/app">
                Open dashboard
                <ArrowRight aria-hidden="true" />
              </a>
            </Button>
            <Button asChild size="lg" variant="ghost">
              <a href="#how-it-works">
                <PlayCircle aria-hidden="true" />
                See how it works
              </a>
            </Button>
          </div>
        </FadeUp>
        <div className="mt-16 grid gap-3 sm:grid-cols-3">
          {[
            ["Tempo testnet", "Treasury-ready"],
            ["USDC / pathUSD", "Token-aware"],
            ["Preview first", "Blocker-safe"],
          ].map(([value, label]) => (
            <div
              className="rounded-lg bg-surface-card p-4 outline outline-[0.5px] outline-surface-border"
              key={label}
            >
              <p className="font-mono text-lg font-semibold text-ink-primary">
                {value}
              </p>
              <p className="mt-1 text-sm font-semibold text-ink-muted">
                {label}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
