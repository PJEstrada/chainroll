import { ArrowRight } from "lucide-react";

import { Button } from "@/components/ui/button";

export function CtaBanner() {
  return (
    <section className="px-6 py-24 lg:px-8 lg:py-32" id="pricing">
      <div className="mx-auto max-w-6xl rounded-lg bg-brand-blush px-6 py-16 outline outline-[0.5px] outline-surface-border lg:px-12">
        <div className="flex flex-col gap-8 lg:flex-row lg:items-center lg:justify-between">
          <div className="max-w-2xl">
            <p className="mb-4 text-sm font-semibold uppercase tracking-normal text-ink-muted">
              Demo-ready capstone
            </p>
            <h2 className="text-4xl font-semibold tracking-tight text-ink-primary">
              Show workers a payroll flow that respects their time.
            </h2>
          </div>
          <Button asChild size="lg">
            <a href="/app">
              Get Started
              <ArrowRight aria-hidden="true" />
            </a>
          </Button>
        </div>
      </div>
    </section>
  );
}
