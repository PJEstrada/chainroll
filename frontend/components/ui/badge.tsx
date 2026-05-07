import { cva, type VariantProps } from "class-variance-authority";
import type * as React from "react";

import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex h-7 items-center rounded-full px-3 text-sm font-semibold",
  {
    variants: {
      variant: {
        default: "bg-surface-page text-ink-secondary",
        success: "bg-success-bg text-success",
        warning: "bg-warning-bg text-warning",
        error: "bg-error-bg text-error",
        outline:
          "bg-transparent text-ink-secondary outline outline-[0.5px] outline-surface-border",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  },
);

function Badge({
  className,
  variant,
  ...props
}: React.ComponentProps<"span"> & VariantProps<typeof badgeVariants>) {
  return (
    <span
      className={cn(badgeVariants({ variant }), className)}
      data-slot="badge"
      {...props}
    />
  );
}

export { Badge, badgeVariants };
