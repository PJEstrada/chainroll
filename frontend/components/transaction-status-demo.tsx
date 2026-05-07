"use client";

import { ArrowRight, CheckCircle2, RefreshCw } from "lucide-react";
import { useMemo, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import type {
  PayrollExecution,
  PayrollExecutionAttempt,
  PayrollPreview,
  PayrollPreviewBlocker,
  PayrollPreviewItem,
} from "@/lib/payroll";
import { mockPayrollClient } from "@/lib/payroll";

type DemoPhase = "idle" | "preview" | "review" | "executed";

const blockerLabels: Record<PayrollPreviewBlocker, string> = {
  missing_wallet: "Missing wallet",
  missing_active_compensation_profile: "Missing active comp",
  missing_treasury_account: "Missing treasury",
  token_mismatch: "Token mismatch",
  treasury_requires_user_signature: "Signature required",
};

const executionSteps = [
  "Preview",
  "Review blockers",
  "Create payrun",
  "Submit attempts",
  "Settlement result",
];

function previewBadgeVariant(item: PayrollPreviewItem) {
  if (item.blockers.includes("treasury_requires_user_signature")) {
    return "warning" as const;
  }

  return item.blockers.length === 0 ? ("success" as const) : ("error" as const);
}

function previewLabel(item: PayrollPreviewItem) {
  if (item.blockers.includes("treasury_requires_user_signature")) {
    return "Review required";
  }

  return item.blockers.length === 0 ? "Payable" : "Blocked";
}

function attemptBadgeVariant(attempt: PayrollExecutionAttempt) {
  if (attempt.status === "completed") {
    return "success" as const;
  }

  return attempt.status === "failed"
    ? ("error" as const)
    : ("warning" as const);
}

function stepVariant(
  index: number,
  phase: DemoPhase,
  execution: PayrollExecution | null,
) {
  if (phase === "idle") {
    return index === 0 ? "warning" : "outline";
  }

  if (phase === "preview") {
    return index === 0 ? "success" : index === 1 ? "warning" : "outline";
  }

  if (phase === "review") {
    return index <= 1 ? "success" : index === 2 ? "warning" : "outline";
  }

  if (!execution) {
    return "outline";
  }

  if (index <= 3) {
    return "success";
  }

  return execution.status === "partially_failed" ||
    execution.status === "failed"
    ? "warning"
    : "success";
}

function formatBlockers(item: PayrollPreviewItem) {
  if (item.blockers.length === 0) {
    return ["Ready"];
  }

  return item.blockers.map((blocker) => blockerLabels[blocker]);
}

export function TransactionStatusDemo() {
  const [preview, setPreview] = useState<PayrollPreview | null>(null);
  const [execution, setExecution] = useState<PayrollExecution | null>(null);
  const [phase, setPhase] = useState<DemoPhase>("idle");
  const [loading, setLoading] = useState(false);

  const payableTotal = useMemo(() => {
    if (!preview) {
      return "0 USDC";
    }

    return preview.totals.totalAmounts
      .map((amount) => amount.display)
      .join(" + ");
  }, [preview]);

  async function handlePrimaryAction() {
    if (loading) {
      return;
    }

    setLoading(true);

    try {
      if (!preview) {
        const nextPreview = await mockPayrollClient.getPayrollPreview({
          tenantId: "000000000003V",
          runDate: "2026-05-15",
        });
        setPreview(nextPreview);
        setPhase("preview");
        return;
      }

      if (phase === "preview" && preview.totals.totalBlockers > 0) {
        setPhase("review");
        return;
      }

      if (!execution) {
        const nextExecution = await mockPayrollClient.executePayroll({
          previewId: preview.id,
          executeBlocked: false,
        });
        setExecution(nextExecution);
        setPhase("executed");
        return;
      }

      const nextExecution = await mockPayrollClient.getPayrollExecutionStatus(
        execution.id,
      );
      setExecution(nextExecution);
    } finally {
      setLoading(false);
    }
  }

  const buttonLabel = !preview
    ? "Preview payroll"
    : phase === "preview" && preview.totals.totalBlockers > 0
      ? "Review blockers"
      : execution
        ? "Refresh status"
        : "Execute ready payments";

  const previewItems = preview?.items ?? [];

  return (
    <section
      className="min-h-[150svh] border-y-[0.5px] border-surface-border bg-surface-page py-28 lg:py-40"
      id="payroll-demo"
    >
      <div className="mx-auto max-w-6xl px-6 lg:px-8">
        <div className="grid gap-8 lg:grid-cols-[0.86fr_1.14fr] lg:items-start">
          <div className="lg:sticky lg:top-28">
            <p className="mb-4 text-sm font-semibold uppercase tracking-normal text-ink-muted">
              Payroll command center
            </p>
            <h2 className="text-4xl font-semibold tracking-tight text-ink-primary">
              Preview blockers before execution starts.
            </h2>
            <p className="mt-4 text-lg leading-relaxed text-ink-secondary">
              The mock client mirrors the backend domain: employees, active
              compensation, default treasury, payout blockers, and settlement
              attempt states.
            </p>

            <div className="mt-8 grid grid-cols-2 gap-3">
              {[
                ["Employees", preview?.totals.totalEmployees ?? "—"],
                ["Payable", payableTotal],
                ["Ready", preview?.totals.employeesReady ?? "—"],
                ["Blockers", preview?.totals.totalBlockers ?? "—"],
              ].map(([label, value]) => (
                <Card className="p-4" key={label}>
                  <p className="text-sm font-semibold text-ink-muted">
                    {label}
                  </p>
                  <p className="mt-2 font-mono text-lg font-semibold text-ink-primary">
                    {value}
                  </p>
                </Card>
              ))}
            </div>

            <Button
              className="mt-8 w-full sm:w-auto"
              disabled={loading}
              onClick={handlePrimaryAction}
              type="button"
              variant="secondary"
            >
              {loading ? (
                <RefreshCw aria-hidden="true" className="animate-spin" />
              ) : execution ? (
                <RefreshCw aria-hidden="true" />
              ) : (
                <ArrowRight aria-hidden="true" />
              )}
              {buttonLabel}
            </Button>
          </div>

          <Card className="overflow-hidden">
            <div className="border-b-[0.5px] border-surface-border p-5">
              <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <div>
                  <p className="text-sm font-semibold text-ink-muted">
                    Preview
                  </p>
                  <h3 className="mt-1 text-xl font-semibold tracking-tight text-ink-primary">
                    May payroll run
                  </h3>
                </div>
                <Badge
                  variant={
                    preview?.status === "ready"
                      ? "success"
                      : preview?.status === "blocked"
                        ? "error"
                        : "warning"
                  }
                >
                  {preview?.status.replace("_", " ") ?? "Awaiting preview"}
                </Badge>
              </div>
            </div>

            <div className="grid gap-3 border-b-[0.5px] border-surface-border p-5 sm:grid-cols-5">
              {executionSteps.map((step, index) => (
                <div className="flex items-center gap-2" key={step}>
                  <Badge variant={stepVariant(index, phase, execution)}>
                    {index + 1}
                  </Badge>
                  <span className="text-sm font-semibold text-ink-secondary">
                    {step}
                  </span>
                </div>
              ))}
            </div>

            <div className="overflow-x-auto">
              <table className="w-full min-w-[780px] border-collapse text-left">
                <thead>
                  <tr className="border-b-[0.5px] border-surface-border text-sm font-semibold text-ink-secondary">
                    <th className="px-5 py-4">Worker</th>
                    <th className="px-5 py-4">Wallet</th>
                    <th className="px-5 py-4">Compensation</th>
                    <th className="px-5 py-4">Treasury</th>
                    <th className="px-5 py-4">Blockers</th>
                    <th className="px-5 py-4">Readiness</th>
                  </tr>
                </thead>
                <tbody>
                  {previewItems.length === 0 ? (
                    <tr>
                      <td
                        className="px-5 py-8 text-base text-ink-secondary"
                        colSpan={6}
                      >
                        Run the preview to load worker readiness.
                      </td>
                    </tr>
                  ) : (
                    previewItems.map((item) => (
                      <tr
                        className="border-b-[0.5px] border-surface-border last:border-b-0"
                        key={item.employeeId}
                      >
                        <td className="px-5 py-4">
                          <p className="font-semibold text-ink-primary">
                            {item.workerName}
                          </p>
                          <p className="mt-1 text-sm font-semibold text-ink-muted">
                            {item.identifier}
                          </p>
                        </td>
                        <td className="px-5 py-4">
                          <Badge
                            variant={item.walletAddress ? "success" : "error"}
                          >
                            {item.walletAddress
                              ? "Wallet linked"
                              : "Missing wallet"}
                          </Badge>
                        </td>
                        <td className="px-5 py-4">
                          <Badge variant={item.amount ? "success" : "error"}>
                            {item.amount
                              ? item.amount.display
                              : "No active profile"}
                          </Badge>
                        </td>
                        <td className="px-5 py-4">
                          <Badge
                            variant={
                              item.treasury?.matchesDefault
                                ? item.treasury.controlMode ===
                                  "user_signature_required"
                                  ? "warning"
                                  : "success"
                                : "error"
                            }
                          >
                            {item.treasury?.matchesDefault
                              ? item.treasury.controlMode.replaceAll("_", " ")
                              : "No token match"}
                          </Badge>
                        </td>
                        <td className="px-5 py-4">
                          <div className="flex flex-wrap gap-2">
                            {formatBlockers(item).map((label) => (
                              <Badge
                                key={label}
                                variant={
                                  label === "Ready"
                                    ? "success"
                                    : previewBadgeVariant(item)
                                }
                              >
                                {label}
                              </Badge>
                            ))}
                          </div>
                        </td>
                        <td className="px-5 py-4">
                          <Badge variant={previewBadgeVariant(item)}>
                            {previewLabel(item)}
                          </Badge>
                        </td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>

            {execution ? (
              <div className="border-t-[0.5px] border-surface-border p-5">
                <div className="mb-4 flex items-center gap-2">
                  <CheckCircle2
                    aria-hidden="true"
                    className="size-5 text-ink-secondary"
                  />
                  <h3 className="text-xl font-semibold tracking-tight text-ink-primary">
                    Execution attempts
                  </h3>
                </div>
                <div className="grid gap-3">
                  {execution.attempts.map((attempt) => (
                    <div
                      className="flex flex-col gap-3 rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border sm:flex-row sm:items-center sm:justify-between"
                      key={attempt.id}
                    >
                      <div>
                        <p className="font-semibold text-ink-primary">
                          {attempt.workerName}
                        </p>
                        <p className="mt-1 font-mono text-sm font-semibold text-ink-muted">
                          {attempt.amount.display}
                        </p>
                      </div>
                      <Badge variant={attemptBadgeVariant(attempt)}>
                        {attempt.label}
                      </Badge>
                    </div>
                  ))}
                </div>
              </div>
            ) : null}
          </Card>
        </div>
      </div>
    </section>
  );
}
