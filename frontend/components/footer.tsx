import { footerColumns } from "@/lib/content";

export function Footer() {
  return (
    <footer
      className="border-t-[0.5px] border-surface-border bg-surface-page"
      id="support"
    >
      <div className="mx-auto max-w-6xl px-6 py-16 lg:px-8">
        <div className="grid gap-10 lg:grid-cols-[1.2fr_3fr]">
          <div>
            <p className="text-2xl font-bold tracking-tight text-brand-primary">
              Boto
            </p>
            <p className="mt-4 max-w-xs text-base leading-relaxed text-ink-secondary">
              Cross-border payroll previews and stablecoin payout readiness for
              teams with workers across borders.
            </p>
          </div>
          <div className="grid gap-8 sm:grid-cols-2 lg:grid-cols-4">
            {footerColumns.map((column) => (
              <div key={column.title}>
                <h3 className="text-sm font-semibold uppercase tracking-normal text-ink-muted">
                  {column.title}
                </h3>
                <ul className="mt-4 space-y-3">
                  {column.links.map((link) => (
                    <li key={link}>
                      <a
                        className="text-sm font-semibold text-ink-secondary transition-colors hover:text-ink-primary"
                        href="#support"
                      >
                        {link}
                      </a>
                    </li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        </div>
        <div className="mt-14 flex flex-col gap-4 border-t-[0.5px] border-surface-border pt-6 text-sm font-semibold text-ink-muted sm:flex-row sm:items-center sm:justify-between">
          <p>&copy; 2026 Boto. All rights reserved.</p>
          <div className="flex gap-5">
            <a href="#support">Privacy</a>
            <a href="#support">Terms</a>
            <a href="#support">Security</a>
          </div>
        </div>
      </div>
    </footer>
  );
}
