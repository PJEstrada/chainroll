import {
  payrollExecutionFixture,
  payrollPreviewFixture,
} from "@/lib/payroll/fixtures";
import type {
  PayrollClient,
  PayrollExecution,
  PayrollExecutionAttempt,
  PayrollExecutionInput,
  PayrollPreview,
  PayrollPreviewInput,
} from "@/lib/payroll/types";

const executionPolls = new Map<string, number>();
const executions = new Map<string, PayrollExecution>();
const transactionHashes = {
  attempt_ana:
    "0x8b2a1c3d4e5f60718293a4b5c6d7e8f90123456789abcdef0123456789abcde",
  attempt_mateo:
    "0x6e9f44a312b0c8d97740edaae21f6b4f56c331a4d25f90189abcdeffedcba123",
};

function clone<T>(value: T): T {
  return globalThis.structuredClone(value);
}

function delay(ms: number) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

function transactionHashFor(seed: string) {
  const hex = Array.from(seed)
    .map((char) => char.charCodeAt(0).toString(16).padStart(2, "0"))
    .join("")
    .padEnd(64, "0")
    .slice(0, 64);

  return `0x${hex}`;
}

function executionFromPreview(preview: PayrollPreview): PayrollExecution {
  const attempts = preview.items
    .filter((item) => item.amount && item.blockers.length === 0)
    .map<PayrollExecutionAttempt>((item) => ({
      id: `attempt_${item.employeeId}`,
      employeeId: item.employeeId,
      workerName: item.workerName,
      amount: item.amount as PayrollExecutionAttempt["amount"],
      status: "queued",
      label: "In transit",
      transactionHash: null,
    }));

  return {
    id: `payrun_${preview.id}`,
    previewId: preview.id,
    status: attempts.length === 0 ? "review_required" : "queued",
    createdAt: new Date().toISOString(),
    readyPaymentCount: attempts.length,
    attempts,
  };
}

function executionSnapshotForPoll(id: string): PayrollExecution {
  const poll = executionPolls.get(id) ?? 0;
  const execution = clone(executions.get(id) ?? payrollExecutionFixture);
  executionPolls.set(id, poll + 1);

  if (execution.attempts.length === 0) {
    return {
      ...execution,
      status: "review_required",
    };
  }

  if (poll === 0) {
    return {
      ...execution,
      status: "submitting",
      attempts: execution.attempts.map((attempt, index) =>
        index === 0
          ? {
              ...attempt,
              status: "submitting",
            }
          : attempt,
      ),
    };
  }

  if (poll === 1) {
    return {
      ...execution,
      status: "submitted",
      attempts: execution.attempts.map((attempt, index) => {
        if (index === 0) {
          return {
            ...attempt,
            status: "completed",
            label: "Payment arrived",
            transactionHash:
              transactionHashes[attempt.id as keyof typeof transactionHashes] ??
              transactionHashFor(attempt.id),
          };
        }

        if (index === 1) {
          return {
            ...attempt,
            status: "submitted",
            transactionHash:
              transactionHashes[attempt.id as keyof typeof transactionHashes] ??
              transactionHashFor(attempt.id),
          };
        }

        return {
          ...attempt,
          status: "submitting",
        };
      }),
    };
  }

  const hasFailedAttempt = execution.attempts.length > 2;

  return {
    ...execution,
    status: hasFailedAttempt ? "partially_failed" : "completed",
    attempts: execution.attempts.map((attempt, index) =>
      hasFailedAttempt && index === execution.attempts.length - 1
        ? {
            ...attempt,
            status: "failed",
            label: "Transfer failed",
          }
        : {
            ...attempt,
            status: "completed",
            label: "Payment arrived",
            transactionHash:
              transactionHashes[attempt.id as keyof typeof transactionHashes] ??
              transactionHashFor(attempt.id),
          },
    ),
  };
}

export const mockPayrollClient: PayrollClient = {
  async getPayrollPreview(input: PayrollPreviewInput): Promise<PayrollPreview> {
    await delay(520);
    return {
      ...clone(payrollPreviewFixture),
      tenantId: input.tenantId,
    };
  },

  async executePayroll(
    input: PayrollExecutionInput,
  ): Promise<PayrollExecution> {
    await delay(680);
    const execution = input.preview
      ? executionFromPreview(input.preview)
      : {
          ...clone(payrollExecutionFixture),
          previewId: input.previewId,
        };
    executions.set(execution.id, execution);
    executionPolls.set(execution.id, 0);
    return execution;
  },

  async getPayrollExecutionStatus(id: string): Promise<PayrollExecution> {
    await delay(420);
    return executionSnapshotForPoll(id);
  },
};
