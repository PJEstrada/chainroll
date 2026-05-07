import { FadeUp } from "@/components/fade-up";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { features } from "@/lib/content";

export function Features() {
  return (
    <section className="py-24 lg:py-32" id="capabilities">
      <div className="mx-auto max-w-6xl px-6 lg:px-8">
        <div className="max-w-2xl">
          <p className="mb-4 text-sm font-semibold uppercase tracking-normal text-ink-muted">
            Backend-aware by design
          </p>
          <h2 className="text-4xl font-semibold tracking-tight text-ink-primary">
            Everything the payroll backend is growing into.
          </h2>
          <p className="mt-4 text-lg leading-relaxed text-ink-secondary">
            The landing page mirrors the real service boundaries already in the
            repo, from employees and compensation to treasury and payrun
            preview.
          </p>
        </div>

        <div className="mt-12 grid gap-4 lg:grid-cols-3">
          {features.map((feature, index) => {
            const Icon = feature.icon;

            return (
              <FadeUp delay={index * 0.04} key={feature.title}>
                <Card className="h-full p-1">
                  <CardHeader>
                    <div className="mb-4 flex size-12 items-center justify-center rounded-lg bg-brand-blush text-brand-primary">
                      <Icon aria-hidden="true" className="size-6" />
                    </div>
                    <CardTitle>{feature.title}</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <CardDescription>{feature.description}</CardDescription>
                  </CardContent>
                </Card>
              </FadeUp>
            );
          })}
        </div>
      </div>
    </section>
  );
}
