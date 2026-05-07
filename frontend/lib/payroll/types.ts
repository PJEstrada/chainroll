export type PayrollPreviewStatus = "ready" | "partially_ready" | "blocked";

export type PayrollPreviewBlocker =
  | "missing_wallet"
  | "missing_active_compensation_profile"
  | "missing_treasury_account"
  | "token_mismatch"
  | "treasury_requires_user_signature";

export type PayrollExecutionStatus =
  | "queued"
  | "review_required"
  | "submitting"
  | "submitted"
  | "partially_failed"
  | "failed"
  | "completed";

export type PayrollAttemptStatus =
  | "queued"
  | "submitting"
  | "submitted"
  | "completed"
  | "failed";

export type TreasuryControlMode =
  | "server_controlled"
  | "user_delegated"
  | "user_signature_required"
  | "external_execution";

export type TokenAmount = {
  amountUnits: string;
  tokenSymbol: string;
  display: string;
};

export type PayrollPreviewInput = {
  tenantId: string;
  runDate: string;
};

export type PayrollExecutionInput = {
  previewId: string;
  preview?: PayrollPreview;
  executeBlocked?: false;
};

export type PayrollPreviewItem = {
  employeeId: string;
  identifier: string;
  workerName: string;
  walletAddress: string | null;
  amount: TokenAmount | null;
  cadence: "weekly" | "biweekly" | "monthly" | "custom";
  cadenceLabel?: string;
  treasury: {
    chain: "tempo-testnet";
    tokenSymbol: string;
    controlMode: TreasuryControlMode;
    matchesDefault: boolean;
  } | null;
  blockers: PayrollPreviewBlocker[];
};

export type PayrollPreviewTotals = {
  totalAmounts: TokenAmount[];
  totalBlockers: number;
  totalEmployees: number;
  employeesReady: number;
  employeesBlocked: number;
};

export type PayrollPreview = {
  id: string;
  tenantId: string;
  source?: "api" | "mock";
  status: PayrollPreviewStatus;
  createdAt: string;
  totals: PayrollPreviewTotals;
  items: PayrollPreviewItem[];
};

export type PayrollExecutionAttempt = {
  id: string;
  employeeId: string;
  workerName: string;
  amount: TokenAmount;
  status: PayrollAttemptStatus;
  label: "Payment arrived" | "In transit" | "Transfer failed";
  transactionHash: string | null;
};

export type PayrollExecution = {
  id: string;
  previewId: string;
  status: PayrollExecutionStatus;
  createdAt: string;
  readyPaymentCount: number;
  attempts: PayrollExecutionAttempt[];
};

export type PayrollClient = {
  getPayrollPreview(input: PayrollPreviewInput): Promise<PayrollPreview>;
  executePayroll(input: PayrollExecutionInput): Promise<PayrollExecution>;
  getPayrollExecutionStatus(id: string): Promise<PayrollExecution>;
};
