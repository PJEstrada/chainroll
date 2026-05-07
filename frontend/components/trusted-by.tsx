import { trustedBy } from "@/lib/content";

export function TrustedBy() {
  return (
    <section className="border-y-[0.5px] border-surface-border bg-surface-page py-10">
      <div className="mx-auto max-w-6xl px-6 lg:px-8">
        <div className="grid gap-3 sm:grid-cols-3 lg:grid-cols-6">
          {trustedBy.map((name) => (
            <div
              className="flex h-14 items-center justify-center rounded-lg bg-surface-card px-4 text-sm font-semibold text-ink-muted outline outline-[0.5px] outline-surface-border"
              key={name}
            >
              {name}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
