import { howItWorksSteps } from "@/lib/content";

export function HowItWorks() {
  return (
    <section
      className="border-y-[0.5px] border-surface-border bg-surface-card py-24 lg:py-32"
      id="how-it-works"
    >
      <div className="mx-auto max-w-6xl px-6 lg:px-8">
        <div className="max-w-2xl">
          <p className="mb-4 text-sm font-semibold uppercase tracking-normal text-ink-muted">
            How it works
          </p>
          <h2 className="text-4xl font-semibold tracking-tight text-ink-primary">
            From roster to ready payments in three checks.
          </h2>
        </div>

        <div className="mt-14 grid gap-8 lg:grid-cols-3 lg:gap-0">
          {howItWorksSteps.map((step, index) => (
            <div className="relative lg:pr-10" key={step.number}>
              {index < howItWorksSteps.length - 1 ? (
                <div className="absolute left-12 top-6 hidden h-px w-[calc(100%-3rem)] bg-surface-border lg:block" />
              ) : null}
              <div className="relative z-10 flex size-12 items-center justify-center rounded-full bg-surface-page font-mono text-sm font-semibold text-brand-primary outline outline-[0.5px] outline-surface-border">
                {step.number}
              </div>
              <h3 className="mt-6 text-xl font-semibold tracking-tight text-ink-primary">
                {step.title}
              </h3>
              <p className="mt-3 text-base leading-relaxed text-ink-secondary">
                {step.description}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
