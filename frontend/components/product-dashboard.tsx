"use client";

import {
  AlertCircle,
  ArrowRight,
  Banknote,
  CheckCircle2,
  ChevronLeft,
  Circle,
  ExternalLink,
  Eye,
  FileClock,
  LayoutDashboard,
  Loader2,
  Plus,
  RefreshCw,
  Settings,
  ShieldCheck,
  Trash2,
  UserPlus,
  Users,
  WalletCards,
} from "lucide-react";
import { motion, useReducedMotion } from "motion/react";
import {
  cloneElement,
  type Dispatch,
  type FormEvent,
  isValidElement,
  type ReactElement,
  type ReactNode,
  type SetStateAction,
  useEffect,
  useId,
  useMemo,
  useState,
} from "react";

import { ThemeToggle } from "@/components/theme-toggle";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import {
  botoApi,
  botoSettingsStorageKey,
  defaultBotoApiSettings,
} from "@/lib/backend/client";
import type {
  BotoApiSettings,
  CompensationProfile,
  CompensationProfileInput,
  Employee,
  TreasuryAccount,
  TreasuryAccountInput,
} from "@/lib/backend/types";
import type {
  PayrollExecution,
  PayrollExecutionAttempt,
  PayrollExecutionStatus,
  PayrollPreview,
  PayrollPreviewBlocker,
  PayrollPreviewItem,
  TokenAmount,
} from "@/lib/payroll";
import { mockPayrollClient } from "@/lib/payroll";
import { cn } from "@/lib/utils";

type AppView = "overview" | "workers" | "treasury" | "payroll" | "settings";
type ConnectionState = "idle" | "checking" | "connected" | "error";
type WorkerStep = "identity" | "wallet" | "compensation" | "review";
type PayrollTab = "preview" | "review" | "confirm" | "execution";
type PayrollMode = "list" | "wizard";

type PayrollDraft = {
  id: string;
  name: string;
  runDate: string;
  note: string;
  selectedEmployeeIds: string[];
};

type PayrollRunRecord = {
  id: string;
  name: string;
  runDate: string;
  createdAt: string;
  status: PayrollExecutionStatus | "draft";
  totalDisplay: string;
  employeeCount: number;
  readyPaymentCount: number;
  blockedCount: number;
  executionId?: string;
};

type FieldProps = {
  label: string;
  children: ReactElement<{ id?: string; "aria-label"?: string }> | ReactNode;
};

type EmployeeWizardForm = {
  identifier: string;
  first_name: string;
  last_name: string;
  wallet_address: string;
  create_compensation: boolean;
  amount_units: string;
  token_symbol: string;
  cadence: CompensationProfileInput["cadence"];
  cadence_every: string;
  cadence_unit: NonNullable<CompensationProfileInput["cadence_unit"]>;
};

type TreasuryForm = {
  name: string;
  chain: TreasuryAccountInput["chain"];
  token_symbol: string;
  token_address: string;
  token_decimals: number;
  sender_address: string;
  custody_provider: TreasuryAccountInput["custody_provider"];
  control_mode: TreasuryAccountInput["control_mode"];
  provider_wallet_id: string;
  provider_owner_id: string;
  secret_reference: string;
  is_default: boolean;
};

type CompensationForm = {
  amount_units: string;
  token_symbol: string;
  cadence: CompensationProfileInput["cadence"];
  cadence_every: string;
  cadence_unit: NonNullable<CompensationProfileInput["cadence_unit"]>;
};

const appNavItems = [
  { id: "overview", label: "Overview", icon: LayoutDashboard },
  { id: "workers", label: "Workers", icon: Users },
  { id: "treasury", label: "Treasury", icon: Banknote },
  { id: "payroll", label: "Payroll", icon: FileClock },
  { id: "settings", label: "Settings", icon: Settings },
] satisfies Array<{ id: AppView; label: string; icon: typeof LayoutDashboard }>;

const workerSteps = [
  { id: "identity", label: "Identity" },
  { id: "wallet", label: "Wallet" },
  { id: "compensation", label: "Pay" },
  { id: "review", label: "Review" },
] satisfies Array<{ id: WorkerStep; label: string }>;

const initialEmployeeWizard: EmployeeWizardForm = {
  identifier: "EMP-008",
  first_name: "Maya",
  last_name: "Lopez",
  wallet_address: "0x1234567890abcdef1234567890abcdef12345678",
  create_compensation: true,
  amount_units: "1000000",
  token_symbol: "USDC",
  cadence: "monthly",
  cadence_every: "",
  cadence_unit: "weeks",
};

const initialTreasuryForm: TreasuryForm = {
  name: "Tempo USDC source",
  chain: "tempo-testnet",
  token_symbol: "USDC",
  token_address: "0x20c0000000000000000000000000000000000000",
  token_decimals: 18,
  sender_address: "0x1234567890abcdef1234567890abcdef12345678",
  custody_provider: "local_key",
  control_mode: "server_controlled",
  provider_wallet_id: "",
  provider_owner_id: "",
  secret_reference: "env:TEMPO_TREASURY_PRIVATE_KEY",
  is_default: true,
};

const initialCompensationForm: CompensationForm = {
  amount_units: "1000000",
  token_symbol: "USDC",
  cadence: "monthly",
  cadence_every: "",
  cadence_unit: "weeks",
};

const initialPayrollRuns: PayrollRunRecord[] = [
  {
    id: "payroll_may_draft",
    name: "May contractor payroll",
    runDate: "2026-05-15",
    createdAt: "2026-05-02",
    status: "draft",
    totalDisplay: "1,000,000 USDC",
    employeeCount: 5,
    readyPaymentCount: 3,
    blockedCount: 2,
  },
  {
    id: "payroll_april_complete",
    name: "April contractor payroll",
    runDate: "2026-04-15",
    createdAt: "2026-04-12",
    status: "completed",
    totalDisplay: "3,250,000 USDC",
    employeeCount: 8,
    readyPaymentCount: 8,
    blockedCount: 0,
    executionId: "payrun_april_complete",
  },
  {
    id: "payroll_march_partial",
    name: "March contractor payroll",
    runDate: "2026-03-15",
    createdAt: "2026-03-12",
    status: "partially_failed",
    totalDisplay: "2,750,000 USDC",
    employeeCount: 7,
    readyPaymentCount: 7,
    blockedCount: 0,
    executionId: "payrun_march_partial",
  },
];

const terminalExecutionStatuses: PayrollExecutionStatus[] = [
  "completed",
  "failed",
  "partially_failed",
  "review_required",
];

const payrollWizardSteps: {
  id: PayrollTab;
  label: string;
  description: string;
}[] = [
  {
    id: "preview",
    label: "Preview",
    description: "Load the backend readiness check",
  },
  {
    id: "review",
    label: "Review",
    description: "Inspect payable workers and blockers",
  },
  {
    id: "confirm",
    label: "Confirm",
    description: "Approve the ready batch",
  },
  {
    id: "execution",
    label: "Execute",
    description: "Track mocked payout attempts",
  },
];

function Field({ label, children }: FieldProps) {
  const id = useId();
  const control = isValidElement<{ id?: string; "aria-label"?: string }>(
    children,
  )
    ? cloneElement(children, {
        "aria-label": label,
        id,
      })
    : children;

  return (
    <label
      className="grid gap-2 text-sm font-semibold text-ink-secondary"
      htmlFor={id}
    >
      <span>{label}</span>
      {control}
    </label>
  );
}

function inputClassName(className?: string) {
  return cn(
    "h-10 rounded-lg bg-surface-page px-3 text-sm font-semibold text-ink-primary outline outline-[0.5px] outline-surface-border placeholder:text-ink-muted",
    className,
  );
}

function selectClassName(className?: string) {
  return cn(
    "h-10 rounded-lg bg-surface-page px-3 text-sm font-semibold text-ink-primary outline outline-[0.5px] outline-surface-border",
    className,
  );
}

function truncateAddress(value: string | null) {
  if (!value) {
    return "Missing";
  }

  return `${value.slice(0, 8)}...${value.slice(-6)}`;
}

function statusVariant(status?: string): "success" | "outline" {
  return status?.toLowerCase() === "active" ? "success" : "outline";
}

function parseAmountUnits(value: string) {
  try {
    return BigInt(value || "0");
  } catch {
    return BigInt(0);
  }
}

function formatAmountUnits(value: string | bigint) {
  const normalized =
    typeof value === "bigint"
      ? value.toString()
      : value.replace(/^0+/, "") || "0";

  return normalized.replace(/\B(?=(\d{3})+(?!\d))/g, ",");
}

function readSettings() {
  if (typeof window === "undefined") {
    return defaultBotoApiSettings();
  }

  try {
    const stored = window.localStorage.getItem(botoSettingsStorageKey);
    if (!stored) {
      return defaultBotoApiSettings();
    }

    return {
      ...defaultBotoApiSettings(),
      ...JSON.parse(stored),
    } as BotoApiSettings;
  } catch {
    return defaultBotoApiSettings();
  }
}

function compensationInputFromForm(
  form: CompensationForm | EmployeeWizardForm,
): CompensationProfileInput {
  return {
    amount_units: form.amount_units,
    token_symbol: form.token_symbol,
    cadence: form.cadence,
    cadence_every:
      form.cadence === "custom" && form.cadence_every
        ? Number(form.cadence_every)
        : null,
    cadence_unit: form.cadence === "custom" ? form.cadence_unit : null,
  };
}

function formatProfile(profile: CompensationProfile) {
  return `${formatAmountUnits(profile.amount.amount_units)} ${
    profile.amount.token_symbol
  }`;
}

function activeCompensationProfiles(profiles: CompensationProfile[]) {
  return profiles.filter(
    (profile) => profile.metadata.status.toLowerCase() === "active",
  );
}

function summarizeCompensationProfiles(profiles: CompensationProfile[]) {
  const totals = new Map<string, bigint>();

  for (const profile of activeCompensationProfiles(profiles)) {
    const token = profile.amount.token_symbol;
    const current = totals.get(token) ?? BigInt(0);
    totals.set(token, current + parseAmountUnits(profile.amount.amount_units));
  }

  const summary = Array.from(totals.entries()).map(
    ([token, amount]) => `${formatAmountUnits(amount)} ${token}`,
  );

  return summary.length > 0 ? summary.join(" + ") : "No active profiles";
}

function formatBlocker(blocker: PayrollPreviewBlocker) {
  return blocker.replaceAll("_", " ");
}

function blockerResolution(blocker: PayrollPreviewBlocker) {
  const labels: Record<PayrollPreviewBlocker, string> = {
    missing_wallet: "Collect and save a wallet address on the worker profile.",
    missing_active_compensation_profile:
      "Create an active compensation profile before the run date.",
    missing_treasury_account: "Add a default treasury account for this token.",
    token_mismatch: "Match the compensation token to an active treasury token.",
    treasury_requires_user_signature:
      "Queue this worker for signer review before submission.",
  };

  return labels[blocker];
}

function itemVariant(
  item: PayrollPreviewItem,
): "success" | "warning" | "error" {
  if (item.blockers.length === 0) {
    return "success";
  }

  return item.blockers.includes("treasury_requires_user_signature")
    ? "warning"
    : "error";
}

function attemptVariant(
  status: PayrollExecutionAttempt["status"],
): "success" | "warning" | "error" {
  if (status === "completed") {
    return "success";
  }

  return status === "failed" ? "error" : "warning";
}

function executionVariant(
  status: PayrollExecutionStatus | undefined,
): "success" | "warning" | "error" | "outline" {
  if (!status) {
    return "outline";
  }

  if (status === "completed") {
    return "success";
  }

  if (status === "failed" || status === "partially_failed") {
    return "error";
  }

  return "warning";
}

function statusLabel(value: string) {
  return value.replaceAll("_", " ");
}

function explorerUrl(hash: string) {
  return `https://explore.tempo.xyz/tx/${hash}`;
}

function isExecutionTerminal(status: PayrollExecutionStatus) {
  return terminalExecutionStatuses.includes(status);
}

function createDefaultPayrollDraft(): PayrollDraft {
  const id = `payroll_${Date.now()}`;

  return {
    id,
    name: "May payroll run",
    runDate: "2026-05-15",
    note: "",
    selectedEmployeeIds: [],
  };
}

function previewReadyItems(preview: PayrollPreview | null) {
  return preview?.items.filter((item) => item.blockers.length === 0) ?? [];
}

function previewBlockedItems(preview: PayrollPreview | null) {
  return preview?.items.filter((item) => item.blockers.length > 0) ?? [];
}

function attemptProgress(status: PayrollExecutionAttempt["status"]) {
  const progress: Record<PayrollExecutionAttempt["status"], number> = {
    queued: 12,
    submitting: 42,
    submitted: 72,
    completed: 100,
    failed: 100,
  };

  return progress[status];
}

function executionProgress(execution: PayrollExecution | null) {
  if (!execution || execution.attempts.length === 0) {
    return 0;
  }

  const total = execution.attempts.reduce(
    (sum, attempt) => sum + attemptProgress(attempt.status),
    0,
  );

  return Math.round(total / execution.attempts.length);
}

function payrollRunVariant(
  status: PayrollRunRecord["status"],
): "success" | "warning" | "error" | "outline" {
  if (status === "draft") {
    return "outline";
  }

  return executionVariant(status);
}

function tokenTotalsFromItems(items: PayrollPreviewItem[]): TokenAmount[] {
  const totals = new Map<string, bigint>();

  for (const item of items) {
    if (!item.amount) {
      continue;
    }

    const current = totals.get(item.amount.tokenSymbol) ?? BigInt(0);
    totals.set(
      item.amount.tokenSymbol,
      current + parseAmountUnits(item.amount.amountUnits),
    );
  }

  return Array.from(totals.entries()).map(([tokenSymbol, amount]) => ({
    amountUnits: amount.toString(),
    tokenSymbol,
    display: `${formatAmountUnits(amount)} ${tokenSymbol}`,
  }));
}

function payableDisplayForItems(items: PayrollPreviewItem[]) {
  const amounts = tokenTotalsFromItems(items).map((amount) => amount.display);
  return amounts.length > 0 ? amounts.join(" + ") : "No payable amount";
}

function selectedReadyItems(
  preview: PayrollPreview | null,
  selectedEmployeeIds: string[],
) {
  const selected = new Set(selectedEmployeeIds);

  return previewReadyItems(preview).filter((item) =>
    selected.has(item.employeeId),
  );
}

function previewForSelectedPayroll(
  preview: PayrollPreview,
  draft: PayrollDraft,
) {
  const items = selectedReadyItems(preview, draft.selectedEmployeeIds);

  return {
    ...preview,
    id: `${preview.id}_${Date.now()}`,
    totals: {
      totalAmounts: tokenTotalsFromItems(items),
      totalBlockers: 0,
      totalEmployees: items.length,
      employeesReady: items.length,
      employeesBlocked: 0,
    },
    items,
  };
}

function payrollRunFromDraft(
  draft: PayrollDraft,
  preview: PayrollPreview,
  execution?: PayrollExecution,
): PayrollRunRecord {
  const ready = selectedReadyItems(preview, draft.selectedEmployeeIds);
  const blocked = previewBlockedItems(preview);

  return {
    id: draft.id,
    name: draft.name,
    runDate: draft.runDate,
    createdAt: new Date().toISOString(),
    status: execution?.status ?? "draft",
    totalDisplay: payableDisplayForItems(ready),
    employeeCount: ready.length + blocked.length,
    readyPaymentCount: ready.length,
    blockedCount: blocked.length,
    executionId: execution?.id,
  };
}

function upsertPayrollRun(runs: PayrollRunRecord[], nextRun: PayrollRunRecord) {
  const existing = runs.some((run) => run.id === nextRun.id);

  if (existing) {
    return runs.map((run) => (run.id === nextRun.id ? nextRun : run));
  }

  return [nextRun, ...runs];
}

export function ProductDashboard() {
  const [view, setView] = useState<AppView>("overview");
  const [settings, setSettings] = useState<BotoApiSettings>(
    defaultBotoApiSettings(),
  );
  const [connection, setConnection] = useState<ConnectionState>("idle");
  const [employees, setEmployees] = useState<Employee[]>([]);
  const [employeeCount, setEmployeeCount] = useState<number | null>(null);
  const [treasuryAccounts, setTreasuryAccounts] = useState<TreasuryAccount[]>(
    [],
  );
  const [selectedEmployeeId, setSelectedEmployeeId] = useState("");
  const [compensationProfiles, setCompensationProfiles] = useState<
    CompensationProfile[]
  >([]);
  const [
    compensationProfilesByEmployeeId,
    setCompensationProfilesByEmployeeId,
  ] = useState<Record<string, CompensationProfile[]>>({});
  const [loadingCompensationProfiles, setLoadingCompensationProfiles] =
    useState(false);
  const [employeeWizard, setEmployeeWizard] = useState(initialEmployeeWizard);
  const [workerStep, setWorkerStep] = useState<WorkerStep>("identity");
  const [workerDrawerOpen, setWorkerDrawerOpen] = useState(false);
  const [workerDetailOpen, setWorkerDetailOpen] = useState(false);
  const [compensationDrawerOpen, setCompensationDrawerOpen] = useState(false);
  const [selectedTreasuryAccountId, setSelectedTreasuryAccountId] =
    useState("");
  const [treasuryDrawerOpen, setTreasuryDrawerOpen] = useState(false);
  const [treasuryDetailOpen, setTreasuryDetailOpen] = useState(false);
  const [treasuryForm, setTreasuryForm] = useState(initialTreasuryForm);
  const [compensationForm, setCompensationForm] = useState(
    initialCompensationForm,
  );
  const [preview, setPreview] = useState<PayrollPreview | null>(null);
  const [execution, setExecution] = useState<PayrollExecution | null>(null);
  const [payrollMode, setPayrollMode] = useState<PayrollMode>("list");
  const [payrollTab, setPayrollTab] = useState<PayrollTab>("preview");
  const [payrollDraft, setPayrollDraft] = useState<PayrollDraft>(() =>
    createDefaultPayrollDraft(),
  );
  const [payrollRuns, setPayrollRuns] =
    useState<PayrollRunRecord[]>(initialPayrollRuns);
  const [syncing, setSyncing] = useState(false);
  const [mutating, setMutating] = useState(false);
  const [payrollBusy, setPayrollBusy] = useState(false);
  const [pollingExecution, setPollingExecution] = useState(false);
  const [message, setMessage] = useState("");

  const selectedEmployee = employees.find(
    (employee) => employee.id === selectedEmployeeId,
  );

  const defaultTreasury = treasuryAccounts.find(
    (account) => account.is_default,
  );
  const selectedTreasuryAccount = treasuryAccounts.find(
    (account) => account.id === selectedTreasuryAccountId,
  );
  const readyWallets = employees.filter(
    (employee) => employee.wallet_address,
  ).length;

  const payrollTotal = useMemo(() => {
    if (!preview) {
      return "Not previewed";
    }

    return preview.totals.totalAmounts
      .map((amount) => amount.display)
      .join(" + ");
  }, [preview]);

  async function refreshData(nextSettings = settings) {
    setSyncing(true);
    setConnection("checking");
    setMessage("");

    try {
      const [nextEmployees, nextCount, nextTreasury] = await Promise.all([
        botoApi.listEmployees(nextSettings),
        botoApi.countEmployees(nextSettings),
        botoApi.listTreasuryAccounts(nextSettings),
      ]);

      setEmployees(nextEmployees);
      setEmployeeCount(nextCount);
      setTreasuryAccounts(nextTreasury);
      setConnection("connected");
      setSelectedEmployeeId((current) => {
        if (
          current &&
          nextEmployees.some((employee) => employee.id === current)
        ) {
          return current;
        }

        return nextEmployees[0]?.id ?? "";
      });
      setSelectedTreasuryAccountId((current) => {
        if (current && nextTreasury.some((account) => account.id === current)) {
          return current;
        }

        return nextTreasury[0]?.id ?? "";
      });
    } catch (error) {
      setConnection("error");
      setMessage(error instanceof Error ? error.message : "API request failed");
    } finally {
      setSyncing(false);
    }
  }

  // biome-ignore lint/correctness/useExhaustiveDependencies: boot once from local storage, then explicit actions own follow-up refreshes.
  useEffect(() => {
    const nextSettings = readSettings();
    setSettings(nextSettings);
    void refreshData(nextSettings);
  }, []);

  useEffect(() => {
    if (!selectedEmployeeId) {
      setCompensationProfiles([]);
      return;
    }

    void botoApi
      .listCompensationProfiles(settings, selectedEmployeeId)
      .then(setCompensationProfiles)
      .catch(() => setCompensationProfiles([]));
  }, [selectedEmployeeId, settings]);

  useEffect(() => {
    let canceled = false;

    if (employees.length === 0) {
      setCompensationProfilesByEmployeeId({});
      setLoadingCompensationProfiles(false);
      return;
    }

    setLoadingCompensationProfiles(true);
    void Promise.all(
      employees.map(async (employee) => {
        try {
          const profiles = await botoApi.listCompensationProfiles(
            settings,
            employee.id,
          );
          return [employee.id, profiles] as const;
        } catch {
          return [employee.id, []] as const;
        }
      }),
    )
      .then((entries) => {
        if (!canceled) {
          setCompensationProfilesByEmployeeId(Object.fromEntries(entries));
        }
      })
      .finally(() => {
        if (!canceled) {
          setLoadingCompensationProfiles(false);
        }
      });

    return () => {
      canceled = true;
    };
  }, [employees, settings]);

  useEffect(() => {
    if (
      view !== "payroll" ||
      !execution ||
      isExecutionTerminal(execution.status)
    ) {
      return;
    }

    let canceled = false;
    const interval = window.setInterval(() => {
      setPollingExecution(true);
      void mockPayrollClient
        .getPayrollExecutionStatus(execution.id)
        .then((nextExecution) => {
          if (!canceled) {
            setExecution(nextExecution);
            setPayrollRuns((current) =>
              current.map((run) =>
                run.executionId === nextExecution.id
                  ? { ...run, status: nextExecution.status }
                  : run,
              ),
            );
          }
        })
        .finally(() => {
          if (!canceled) {
            setPollingExecution(false);
          }
        });
    }, 1800);

    return () => {
      canceled = true;
      window.clearInterval(interval);
    };
  }, [execution, view]);

  function persistSettings(nextSettings: BotoApiSettings) {
    setSettings(nextSettings);
    window.localStorage.setItem(
      botoSettingsStorageKey,
      JSON.stringify(nextSettings),
    );
    void refreshData(nextSettings);
  }

  async function createWorkerFromWizard(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (workerStep !== "review") {
      setWorkerStep(
        workerSteps[
          Math.min(
            workerSteps.findIndex((step) => step.id === workerStep) + 1,
            workerSteps.length - 1,
          )
        ].id,
      );
      return;
    }

    setMutating(true);
    setMessage("");

    try {
      const employee = await botoApi.createEmployee(settings, {
        identifier: employeeWizard.identifier,
        first_name: employeeWizard.first_name,
        last_name: employeeWizard.last_name,
        wallet_address: employeeWizard.wallet_address || null,
      });

      let profileCreated = false;
      if (employeeWizard.create_compensation) {
        try {
          await botoApi.createCompensationProfile(
            settings,
            employee.id,
            compensationInputFromForm(employeeWizard),
          );
          profileCreated = true;
        } catch (error) {
          setMessage(
            error instanceof Error
              ? `Worker created. Compensation failed: ${error.message}`
              : "Worker created. Compensation failed.",
          );
        }
      }

      setSelectedEmployeeId(employee.id);
      setWorkerDetailOpen(true);
      setEmployeeWizard(initialEmployeeWizard);
      setWorkerStep("identity");
      setWorkerDrawerOpen(false);
      await refreshData();
      if (!employeeWizard.create_compensation || profileCreated) {
        setMessage("Worker created through the Rust API.");
      }
    } catch (error) {
      setMessage(
        error instanceof Error ? error.message : "Unable to create worker",
      );
    } finally {
      setMutating(false);
    }
  }

  async function submitTreasury(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setMutating(true);
    setMessage("");

    try {
      const account = await botoApi.createTreasuryAccount(settings, {
        ...treasuryForm,
        provider_wallet_id: treasuryForm.provider_wallet_id || null,
        provider_owner_id: treasuryForm.provider_owner_id || null,
        secret_reference: treasuryForm.secret_reference || null,
      });
      setSelectedTreasuryAccountId(account.id);
      setTreasuryDetailOpen(true);
      setTreasuryDrawerOpen(false);
      setTreasuryForm(initialTreasuryForm);
      setMessage("Treasury account created through the Rust API.");
      await refreshData();
    } catch (error) {
      setMessage(
        error instanceof Error
          ? error.message
          : "Unable to create treasury account",
      );
    } finally {
      setMutating(false);
    }
  }

  async function submitCompensation(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!selectedEmployeeId) {
      setMessage("Choose a worker before creating compensation.");
      return;
    }

    setMutating(true);
    setMessage("");

    try {
      await botoApi.createCompensationProfile(
        settings,
        selectedEmployeeId,
        compensationInputFromForm(compensationForm),
      );
      const profiles = await botoApi.listCompensationProfiles(
        settings,
        selectedEmployeeId,
      );
      setCompensationProfiles(profiles);
      setCompensationProfilesByEmployeeId((current) => ({
        ...current,
        [selectedEmployeeId]: profiles,
      }));
      setCompensationDrawerOpen(false);
      setCompensationForm(initialCompensationForm);
      setMessage("Compensation profile created through the Rust API.");
    } catch (error) {
      setMessage(
        error instanceof Error
          ? error.message
          : "Unable to create compensation profile",
      );
    } finally {
      setMutating(false);
    }
  }

  async function deleteEmployee(id: string) {
    setMutating(true);
    setMessage("");

    try {
      await botoApi.deleteEmployee(settings, id);
      if (id === selectedEmployeeId) {
        setWorkerDetailOpen(false);
      }
      setCompensationProfilesByEmployeeId((current) => {
        const next = { ...current };
        delete next[id];
        return next;
      });
      setMessage("Worker deleted through the Rust API.");
      await refreshData();
    } catch (error) {
      setMessage(
        error instanceof Error ? error.message : "Unable to delete worker",
      );
    } finally {
      setMutating(false);
    }
  }

  async function deactivateTreasuryAccount(id: string) {
    setMutating(true);
    setMessage("");

    try {
      await botoApi.deactivateTreasuryAccount(settings, id);
      if (id === selectedTreasuryAccountId) {
        setTreasuryDetailOpen(false);
      }
      setMessage("Treasury account deactivated through the Rust API.");
      await refreshData();
    } catch (error) {
      setMessage(
        error instanceof Error
          ? error.message
          : "Unable to deactivate treasury account",
      );
    } finally {
      setMutating(false);
    }
  }

  async function loadPayrollPreview(
    nextStep: PayrollTab = "review",
    draftOverride?: PayrollDraft,
  ) {
    setPayrollBusy(true);
    setMessage("");

    try {
      const nextPreview = await botoApi.previewPayrun(settings, employees);
      const nextDraft = {
        ...(draftOverride ?? payrollDraft),
        selectedEmployeeIds: previewReadyItems(nextPreview).map(
          (item) => item.employeeId,
        ),
      };
      setPreview(nextPreview);
      setExecution(null);
      setPayrollDraft(nextDraft);
      setPayrollRuns((current) =>
        upsertPayrollRun(current, payrollRunFromDraft(nextDraft, nextPreview)),
      );
      setPayrollMode("wizard");
      setPayrollTab(nextStep);
      setMessage("Payroll preview loaded from the Rust API.");
    } catch (error) {
      setMessage(
        error instanceof Error ? error.message : "Unable to preview payroll",
      );
    } finally {
      setPayrollBusy(false);
    }
  }

  function createPayrollRun() {
    const nextDraft = createDefaultPayrollDraft();
    setPayrollDraft(nextDraft);
    setPreview(null);
    setExecution(null);
    setPayrollMode("wizard");
    setPayrollTab("review");
    void loadPayrollPreview("review", nextDraft);
  }

  async function executeReadyPayroll() {
    if (!preview) {
      return;
    }

    const selectedPreview = previewForSelectedPayroll(preview, payrollDraft);

    setPayrollBusy(true);
    setMessage("");

    try {
      const nextExecution = await mockPayrollClient.executePayroll({
        previewId: selectedPreview.id,
        preview: selectedPreview,
        executeBlocked: false,
      });
      setExecution(nextExecution);
      setPayrollRuns((current) =>
        upsertPayrollRun(
          current,
          payrollRunFromDraft(payrollDraft, preview, nextExecution),
        ),
      );
      setPayrollTab("execution");
      setMessage("Mocked execution started for ready payments only.");
    } finally {
      setPayrollBusy(false);
    }
  }

  const pageTitle = appNavItems.find((item) => item.id === view)?.label;

  return (
    <main className="min-h-screen bg-surface-page text-ink-primary">
      <div className="grid min-h-screen lg:grid-cols-[280px_1fr]">
        <aside className="border-b-[0.5px] border-surface-border bg-surface-card lg:border-b-0 lg:border-r-[0.5px]">
          <div className="flex h-full flex-col gap-6 p-5">
            <div className="flex items-center justify-between">
              <a
                className="text-2xl font-bold tracking-tight text-brand-primary"
                href="/"
              >
                Boto
              </a>
              <ThemeToggle />
            </div>

            <nav className="grid gap-2" data-testid="product-sidebar">
              {appNavItems.map((item) => {
                const Icon = item.icon;
                return (
                  <button
                    className={cn(
                      "flex h-11 items-center gap-3 rounded-lg px-3 text-left text-sm font-semibold transition-colors",
                      view === item.id
                        ? "bg-surface-page text-ink-primary outline outline-[0.5px] outline-surface-border"
                        : "text-ink-secondary hover:bg-surface-page hover:text-ink-primary",
                    )}
                    key={item.id}
                    onClick={() => setView(item.id)}
                    type="button"
                  >
                    <Icon aria-hidden="true" className="size-4" />
                    {item.label}
                  </button>
                );
              })}
            </nav>

            <footer className="mt-auto rounded-lg bg-surface-page p-3 outline outline-[0.5px] outline-surface-border">
              <div className="flex items-center justify-between gap-2">
                <Badge
                  className="h-6 px-2 text-sm"
                  variant={
                    connection === "connected"
                      ? "success"
                      : connection === "error"
                        ? "error"
                        : "warning"
                  }
                >
                  API{" "}
                  {connection === "connected"
                    ? "live"
                    : connection === "checking"
                      ? "checking"
                      : "setup"}
                </Badge>
                <Button
                  className="size-8"
                  disabled={syncing}
                  onClick={() => void refreshData()}
                  size="icon"
                  type="button"
                  variant="ghost"
                >
                  <RefreshCw
                    aria-hidden="true"
                    className={cn(syncing && "animate-spin")}
                  />
                  <span className="sr-only">Sync backend</span>
                </Button>
              </div>
              <p className="mt-2 truncate font-mono text-sm font-semibold text-ink-muted">
                {settings.backendUrl}
              </p>
              <p className="mt-1 truncate font-mono text-sm font-semibold text-ink-muted">
                tenant {settings.tenantId}
              </p>
            </footer>
          </div>
        </aside>

        <section className="min-w-0">
          <header className="sticky top-0 z-30 border-b-[0.5px] border-surface-border bg-surface-page/90 px-6 py-4 backdrop-blur lg:px-8">
            <div className="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
              <div>
                <div className="flex flex-wrap items-center gap-2">
                  <p className="text-sm font-semibold uppercase tracking-normal text-ink-muted">
                    Product workspace
                  </p>
                  <Badge
                    variant={connection === "connected" ? "success" : "warning"}
                  >
                    {connection === "connected" ? "Real API" : "Configure API"}
                  </Badge>
                </div>
                <h1 className="mt-1 text-3xl font-semibold tracking-tight text-ink-primary">
                  {pageTitle}
                </h1>
              </div>
              <div className="flex flex-wrap items-center gap-3">
                {message ? (
                  <Badge variant={connection === "error" ? "error" : "outline"}>
                    {message}
                  </Badge>
                ) : null}
                <Button
                  disabled={syncing}
                  onClick={() => void refreshData()}
                  type="button"
                  variant="secondary"
                >
                  <RefreshCw
                    aria-hidden="true"
                    className={cn(syncing && "animate-spin")}
                  />
                  Sync API
                </Button>
              </div>
            </div>
          </header>

          <div className="grid gap-6 p-6 lg:p-8">
            {view !== "settings" ? (
              <ConnectionStrip
                connection={connection}
                message={message}
                setView={setView}
                settings={settings}
                syncing={syncing}
                testConnection={() => void refreshData()}
              />
            ) : null}

            {view === "overview" ? (
              <OverviewView
                connection={connection}
                defaultTreasury={defaultTreasury}
                employeeCount={employeeCount}
                employees={employees}
                payrollTotal={payrollTotal}
                readyWallets={readyWallets}
                setView={setView}
                treasuryAccounts={treasuryAccounts}
              />
            ) : null}

            {view === "workers" ? (
              <WorkersView
                busy={mutating}
                compensationDrawerOpen={compensationDrawerOpen}
                compensationForm={compensationForm}
                compensationProfiles={compensationProfiles}
                compensationProfilesByEmployeeId={
                  compensationProfilesByEmployeeId
                }
                createWorkerFromWizard={createWorkerFromWizard}
                deleteEmployee={deleteEmployee}
                employeeWizard={employeeWizard}
                employees={employees}
                loadingCompensationProfiles={loadingCompensationProfiles}
                selectedEmployee={selectedEmployee}
                setCompensationDrawerOpen={setCompensationDrawerOpen}
                setCompensationForm={setCompensationForm}
                setEmployeeWizard={setEmployeeWizard}
                setSelectedEmployeeId={setSelectedEmployeeId}
                setWorkerDetailOpen={setWorkerDetailOpen}
                setWorkerDrawerOpen={setWorkerDrawerOpen}
                setWorkerStep={setWorkerStep}
                submitCompensation={submitCompensation}
                workerDetailOpen={workerDetailOpen}
                workerDrawerOpen={workerDrawerOpen}
                workerStep={workerStep}
              />
            ) : null}

            {view === "treasury" ? (
              <TreasuryView
                busy={mutating}
                deactivateTreasuryAccount={deactivateTreasuryAccount}
                selectedTreasuryAccount={selectedTreasuryAccount}
                setSelectedTreasuryAccountId={setSelectedTreasuryAccountId}
                setTreasuryForm={setTreasuryForm}
                setTreasuryDetailOpen={setTreasuryDetailOpen}
                setTreasuryDrawerOpen={setTreasuryDrawerOpen}
                submitTreasury={submitTreasury}
                treasuryAccounts={treasuryAccounts}
                treasuryDetailOpen={treasuryDetailOpen}
                treasuryDrawerOpen={treasuryDrawerOpen}
                treasuryForm={treasuryForm}
              />
            ) : null}

            {view === "payroll" ? (
              <PayrollWorkspace
                createPayrollRun={createPayrollRun}
                executeReadyPayroll={executeReadyPayroll}
                execution={execution}
                payrollDraft={payrollDraft}
                payrollBusy={payrollBusy}
                payrollMode={payrollMode}
                payrollRuns={payrollRuns}
                payrollTab={payrollTab}
                pollingExecution={pollingExecution}
                preview={preview}
                refreshPreview={loadPayrollPreview}
                setPayrollDraft={setPayrollDraft}
                setPayrollMode={setPayrollMode}
                setPayrollTab={setPayrollTab}
              />
            ) : null}

            {view === "settings" ? (
              <SettingsView
                connection={connection}
                persistSettings={persistSettings}
                settings={settings}
                syncing={syncing}
                testConnection={() => void refreshData()}
              />
            ) : null}
          </div>
        </section>
      </div>
    </main>
  );
}

function ConnectionStrip({
  connection,
  message,
  setView,
  settings,
  syncing,
  testConnection,
}: {
  connection: ConnectionState;
  message: string;
  setView(view: AppView): void;
  settings: BotoApiSettings;
  syncing: boolean;
  testConnection(): void;
}) {
  if (connection === "connected") {
    return null;
  }

  return (
    <Card className="grid gap-4 p-4 xl:grid-cols-[1fr_auto] xl:items-center">
      <div className="flex items-start gap-3">
        <AlertCircle
          aria-hidden="true"
          className={cn(
            "mt-0.5 size-5",
            connection === "error" ? "text-error" : "text-warning",
          )}
        />
        <div>
          <p className="font-semibold text-ink-primary">
            Backend connection needs attention
          </p>
          <p className="mt-1 text-base leading-relaxed text-ink-secondary">
            The dashboard is wired to the Rust endpoints through the Next proxy.
            Current backend URL:{" "}
            <span className="font-mono text-sm font-semibold text-ink-primary">
              {settings.backendUrl}
            </span>
          </p>
          {message ? (
            <p className="mt-2 text-sm font-semibold text-ink-muted">
              {message}
            </p>
          ) : null}
        </div>
      </div>
      <div className="flex flex-wrap gap-3">
        <Button
          disabled={syncing}
          onClick={testConnection}
          type="button"
          variant="secondary"
        >
          <RefreshCw
            aria-hidden="true"
            className={cn(syncing && "animate-spin")}
          />
          Test connection
        </Button>
        <Button
          onClick={() => setView("settings")}
          type="button"
          variant="outline"
        >
          <Settings aria-hidden="true" />
          Settings
        </Button>
      </div>
    </Card>
  );
}

function OverviewView({
  connection,
  defaultTreasury,
  employeeCount,
  employees,
  payrollTotal,
  readyWallets,
  setView,
  treasuryAccounts,
}: {
  connection: ConnectionState;
  defaultTreasury?: TreasuryAccount;
  employeeCount: number | null;
  employees: Employee[];
  payrollTotal: string;
  readyWallets: number;
  setView(view: AppView): void;
  treasuryAccounts: TreasuryAccount[];
}) {
  const stats: Array<{
    label: string;
    value: ReactNode;
    icon: typeof Users;
  }> = [
    {
      label: "Employees",
      value: employeeCount ?? employees.length,
      icon: Users,
    },
    {
      label: "Wallet-ready",
      value: `${readyWallets}/${employees.length}`,
      icon: WalletCards,
    },
    {
      label: "Treasury",
      value: treasuryAccounts.length,
      icon: Banknote,
    },
    { label: "Preview total", value: payrollTotal, icon: FileClock },
  ];

  return (
    <>
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        {stats.map(({ icon: Icon, label, value }) => (
          <Card className="p-5" key={label}>
            <div className="flex items-center justify-between">
              <p className="text-sm font-semibold text-ink-muted">{label}</p>
              <Icon aria-hidden="true" className="size-5 text-ink-secondary" />
            </div>
            <p className="mt-5 font-mono text-2xl font-semibold text-ink-primary">
              {value}
            </p>
          </Card>
        ))}
      </div>

      <div className="grid gap-6 xl:grid-cols-[1.1fr_0.9fr]">
        <Card className="p-6">
          <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
            <div>
              <h2 className="text-xl font-semibold tracking-tight">
                Payroll readiness
              </h2>
              <p className="mt-2 text-base leading-relaxed text-ink-secondary">
                Worker, wallet, treasury, and preview state before the next run.
              </p>
            </div>
            <Badge variant={connection === "connected" ? "success" : "warning"}>
              {connection === "connected" ? "API live" : "Setup required"}
            </Badge>
          </div>
          <div className="mt-6 grid gap-3">
            {[
              ["Workers loaded", employeeCount ?? employees.length],
              ["Wallet coverage", `${readyWallets}/${employees.length}`],
              ["Default treasury", defaultTreasury?.name ?? "Not configured"],
              ["Payrun preview", "Live Rust API with mocked execution adapter"],
            ].map(([label, value]) => (
              <div
                className="flex flex-col gap-2 rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border sm:flex-row sm:items-center sm:justify-between"
                key={label}
              >
                <p className="text-sm font-semibold text-ink-secondary">
                  {label}
                </p>
                <p className="font-mono text-sm font-semibold text-ink-primary">
                  {value}
                </p>
              </div>
            ))}
          </div>
          <div className="mt-5 flex flex-wrap gap-3">
            <Button
              onClick={() => setView("workers")}
              type="button"
              variant="secondary"
            >
              <Users aria-hidden="true" />
              Open workers
            </Button>
            <Button
              onClick={() => setView("payroll")}
              type="button"
              variant="outline"
            >
              <FileClock aria-hidden="true" />
              Open payroll
            </Button>
          </div>
        </Card>

        <Card className="p-6">
          <h2 className="text-xl font-semibold tracking-tight">
            Recent workers
          </h2>
          <div className="mt-6 grid gap-3">
            {employees.slice(0, 5).map((employee) => (
              <div
                className="flex items-center justify-between gap-4 rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border"
                key={employee.id}
              >
                <div>
                  <p className="font-semibold text-ink-primary">
                    {employee.first_name} {employee.last_name}
                  </p>
                  <p className="mt-1 font-mono text-sm font-semibold text-ink-muted">
                    {employee.identifier}
                  </p>
                </div>
                <Badge
                  variant={employee.wallet_address ? "success" : "warning"}
                >
                  {employee.wallet_address ? "Wallet ready" : "Missing wallet"}
                </Badge>
              </div>
            ))}
            {employees.length === 0 ? (
              <p className="text-base leading-relaxed text-ink-secondary">
                No workers loaded yet.
              </p>
            ) : null}
          </div>
        </Card>
      </div>
    </>
  );
}

function WorkersView({
  busy,
  compensationDrawerOpen,
  compensationForm,
  compensationProfiles,
  compensationProfilesByEmployeeId,
  createWorkerFromWizard,
  deleteEmployee,
  employeeWizard,
  employees,
  loadingCompensationProfiles,
  selectedEmployee,
  setCompensationDrawerOpen,
  setCompensationForm,
  setEmployeeWizard,
  setSelectedEmployeeId,
  setWorkerDetailOpen,
  setWorkerDrawerOpen,
  setWorkerStep,
  submitCompensation,
  workerDetailOpen,
  workerDrawerOpen,
  workerStep,
}: {
  busy: boolean;
  compensationDrawerOpen: boolean;
  compensationForm: CompensationForm;
  compensationProfiles: CompensationProfile[];
  compensationProfilesByEmployeeId: Record<string, CompensationProfile[]>;
  createWorkerFromWizard(event: FormEvent<HTMLFormElement>): Promise<void>;
  deleteEmployee(id: string): Promise<void>;
  employeeWizard: EmployeeWizardForm;
  employees: Employee[];
  loadingCompensationProfiles: boolean;
  selectedEmployee?: Employee;
  setCompensationDrawerOpen(open: boolean): void;
  setCompensationForm: Dispatch<SetStateAction<CompensationForm>>;
  setEmployeeWizard: Dispatch<SetStateAction<EmployeeWizardForm>>;
  setSelectedEmployeeId(value: string): void;
  setWorkerDetailOpen(open: boolean): void;
  setWorkerDrawerOpen(open: boolean): void;
  setWorkerStep(step: WorkerStep): void;
  submitCompensation(event: FormEvent<HTMLFormElement>): Promise<void>;
  workerDetailOpen: boolean;
  workerDrawerOpen: boolean;
  workerStep: WorkerStep;
}) {
  const activeWorkers = employees.filter(
    (employee) => employee.metadata.status.toLowerCase() === "active",
  ).length;
  const walletReady = employees.filter(
    (employee) => employee.wallet_address,
  ).length;
  const allCompensationProfiles = Object.values(
    compensationProfilesByEmployeeId,
  ).flat();
  const activeProfiles = activeCompensationProfiles(allCompensationProfiles);
  const workersWithActiveProfiles = new Set(
    activeProfiles.map((profile) => profile.employee_id),
  ).size;

  const openWorkerProfile = (employee: Employee) => {
    setSelectedEmployeeId(employee.id);
    setWorkerDetailOpen(true);
  };

  const sheets = (
    <>
      <CreateWorkerSheet
        busy={busy}
        employeeWizard={employeeWizard}
        onSubmit={createWorkerFromWizard}
        open={workerDrawerOpen}
        setEmployeeWizard={setEmployeeWizard}
        setOpen={setWorkerDrawerOpen}
        setStep={setWorkerStep}
        step={workerStep}
      />

      <CompensationSheet
        busy={busy}
        compensationForm={compensationForm}
        onSubmit={submitCompensation}
        open={compensationDrawerOpen}
        selectedEmployee={selectedEmployee}
        setCompensationForm={setCompensationForm}
        setOpen={setCompensationDrawerOpen}
      />
    </>
  );

  if (workerDetailOpen && selectedEmployee) {
    return (
      <div className="grid gap-6">
        <WorkerDetailView
          busy={busy}
          compensationProfiles={compensationProfiles}
          deleteEmployee={deleteEmployee}
          loadingCompensationProfiles={loadingCompensationProfiles}
          selectedEmployee={selectedEmployee}
          setCompensationDrawerOpen={setCompensationDrawerOpen}
          setWorkerDetailOpen={setWorkerDetailOpen}
        />
        {sheets}
      </div>
    );
  }

  return (
    <div className="grid gap-6">
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        {[
          {
            label: "Total workers",
            value: employees.length,
            detail: "Current roster size",
          },
          {
            label: "Active",
            value: activeWorkers,
            detail: "Eligible worker records",
          },
          {
            label: "Wallet-ready",
            value: `${walletReady}/${employees.length}`,
            detail: "Can receive payouts",
          },
          {
            label: "Payroll cost",
            value: loadingCompensationProfiles
              ? "Loading..."
              : summarizeCompensationProfiles(allCompensationProfiles),
            detail: `${workersWithActiveProfiles}/${employees.length} with active profiles`,
          },
        ].map((metric) => (
          <Card className="p-5" key={metric.label}>
            <p className="text-sm font-semibold text-ink-muted">
              {metric.label}
            </p>
            <p className="mt-4 break-words font-mono text-xl font-semibold text-ink-primary">
              {metric.value}
            </p>
            <p className="mt-2 text-sm font-semibold text-ink-muted">
              {metric.detail}
            </p>
          </Card>
        ))}
      </div>

      <Card className="overflow-hidden">
        <div className="flex flex-col gap-4 border-b-[0.5px] border-surface-border p-5 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <h2 className="text-xl font-semibold tracking-tight">
              Worker roster
            </h2>
          </div>
          <Button
            onClick={() => {
              setWorkerStep("identity");
              setWorkerDrawerOpen(true);
            }}
            type="button"
          >
            <UserPlus aria-hidden="true" />
            Add worker
          </Button>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full min-w-[860px] text-left">
            <thead>
              <tr className="border-b-[0.5px] border-surface-border text-sm font-semibold text-ink-secondary">
                <th className="px-5 py-4">Worker</th>
                <th className="px-5 py-4">Identifier</th>
                <th className="px-5 py-4">Wallet</th>
                <th className="px-5 py-4">Status</th>
                <th className="px-5 py-4">Compensation</th>
                <th className="px-5 py-4">Actions</th>
              </tr>
            </thead>
            <tbody>
              {employees.map((employee) => {
                const employeeProfiles =
                  compensationProfilesByEmployeeId[employee.id] ?? [];
                const employeeActiveProfiles =
                  activeCompensationProfiles(employeeProfiles);
                const primaryProfile = employeeActiveProfiles[0];
                const profilesLoaded =
                  employee.id in compensationProfilesByEmployeeId;

                return (
                  <tr
                    className="border-b-[0.5px] border-surface-border last:border-b-0 hover:bg-surface-page"
                    key={employee.id}
                  >
                    <td className="px-5 py-4">
                      <button
                        className="grid gap-1 text-left hover:text-brand-primary"
                        onClick={() => openWorkerProfile(employee)}
                        type="button"
                      >
                        <span className="font-semibold text-ink-primary">
                          {employee.first_name} {employee.last_name}
                        </span>
                        <span className="text-sm font-semibold text-ink-muted">
                          View worker profile
                        </span>
                      </button>
                    </td>
                    <td className="px-5 py-4 font-mono text-sm font-semibold text-ink-muted">
                      {employee.identifier}
                    </td>
                    <td className="px-5 py-4">
                      <Badge
                        variant={
                          employee.wallet_address ? "success" : "warning"
                        }
                      >
                        {truncateAddress(employee.wallet_address)}
                      </Badge>
                    </td>
                    <td className="px-5 py-4">
                      <Badge variant={statusVariant(employee.metadata.status)}>
                        {employee.metadata.status}
                      </Badge>
                    </td>
                    <td className="px-5 py-4">
                      {loadingCompensationProfiles && !profilesLoaded ? (
                        <Badge variant="outline">Loading profiles</Badge>
                      ) : primaryProfile ? (
                        <button
                          className="grid gap-1 text-left"
                          onClick={() => openWorkerProfile(employee)}
                          type="button"
                        >
                          <span className="font-mono text-sm font-semibold text-ink-primary hover:text-brand-primary">
                            {formatProfile(primaryProfile)}
                          </span>
                          {employeeActiveProfiles.length > 1 ? (
                            <span className="text-sm font-semibold text-ink-muted">
                              +{employeeActiveProfiles.length - 1} more
                            </span>
                          ) : null}
                        </button>
                      ) : (
                        <Badge variant="warning">Missing profile</Badge>
                      )}
                    </td>
                    <td className="px-5 py-4">
                      <div className="flex flex-wrap gap-2">
                        <Button
                          onClick={() => openWorkerProfile(employee)}
                          size="sm"
                          type="button"
                          variant="secondary"
                        >
                          <Eye aria-hidden="true" />
                          Profile
                        </Button>
                        <Button
                          disabled={busy}
                          onClick={() => void deleteEmployee(employee.id)}
                          size="sm"
                          type="button"
                          variant="outline"
                        >
                          <Trash2 aria-hidden="true" />
                          Delete
                        </Button>
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
          {employees.length === 0 ? (
            <p className="p-5 text-base leading-relaxed text-ink-secondary">
              No workers yet.
            </p>
          ) : null}
        </div>
      </Card>

      {sheets}
    </div>
  );
}

function WorkerDetailView({
  busy,
  compensationProfiles,
  deleteEmployee,
  loadingCompensationProfiles,
  selectedEmployee,
  setCompensationDrawerOpen,
  setWorkerDetailOpen,
}: {
  busy: boolean;
  compensationProfiles: CompensationProfile[];
  deleteEmployee(id: string): Promise<void>;
  loadingCompensationProfiles: boolean;
  selectedEmployee: Employee;
  setCompensationDrawerOpen(open: boolean): void;
  setWorkerDetailOpen(open: boolean): void;
}) {
  const activeProfiles = activeCompensationProfiles(compensationProfiles);

  return (
    <>
      <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
        <Button
          onClick={() => setWorkerDetailOpen(false)}
          type="button"
          variant="ghost"
        >
          <ChevronLeft aria-hidden="true" />
          Workers
        </Button>
        <div className="flex flex-wrap gap-3">
          <Button
            onClick={() => setCompensationDrawerOpen(true)}
            type="button"
            variant="secondary"
          >
            <Plus aria-hidden="true" />
            Add profile
          </Button>
          <Button
            disabled={busy}
            onClick={() => void deleteEmployee(selectedEmployee.id)}
            type="button"
            variant="outline"
          >
            <Trash2 aria-hidden="true" />
            Delete worker
          </Button>
        </div>
      </div>

      <Card className="p-6">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <h2 className="text-3xl font-semibold tracking-tight">
              {selectedEmployee.first_name} {selectedEmployee.last_name}
            </h2>
            <p className="mt-2 font-mono text-sm font-semibold text-ink-muted">
              {selectedEmployee.identifier}
            </p>
          </div>
          <Badge
            variant={selectedEmployee.wallet_address ? "success" : "warning"}
          >
            {selectedEmployee.wallet_address ? "Wallet ready" : "No wallet"}
          </Badge>
        </div>

        <div className="mt-6 grid gap-4 md:grid-cols-3">
          {[
            {
              label: "Payroll cost",
              value: summarizeCompensationProfiles(compensationProfiles),
            },
            {
              label: "Active profiles",
              value: activeProfiles.length,
            },
            {
              label: "Lifecycle",
              value: selectedEmployee.metadata.status,
            },
          ].map((metric) => (
            <div
              className="rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border"
              key={metric.label}
            >
              <p className="text-sm font-semibold text-ink-muted">
                {metric.label}
              </p>
              <p className="mt-3 break-words font-mono text-lg font-semibold text-ink-primary">
                {metric.value}
              </p>
            </div>
          ))}
        </div>
      </Card>

      <div className="grid gap-6 xl:grid-cols-[0.9fr_1.1fr]">
        <Card className="p-5">
          <h2 className="text-xl font-semibold tracking-tight">
            Worker details
          </h2>
          <div className="mt-5 grid gap-3">
            {[
              ["Worker ID", selectedEmployee.id],
              ["Identifier", selectedEmployee.identifier],
              ["Wallet", truncateAddress(selectedEmployee.wallet_address)],
              ["Status", selectedEmployee.metadata.status],
            ].map(([label, value]) => (
              <div
                className="rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border"
                key={label}
              >
                <p className="text-sm font-semibold text-ink-muted">{label}</p>
                <p className="mt-2 break-all font-mono text-sm font-semibold text-ink-primary">
                  {value}
                </p>
              </div>
            ))}
          </div>
        </Card>

        <Card className="p-5">
          <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
            <div>
              <h2 className="text-xl font-semibold tracking-tight">
                Compensation profiles
              </h2>
              <p className="mt-2 text-base leading-relaxed text-ink-secondary">
                Active profiles feed the payroll preview totals.
              </p>
            </div>
            <Button
              onClick={() => setCompensationDrawerOpen(true)}
              type="button"
              variant="secondary"
            >
              <Plus aria-hidden="true" />
              Add profile
            </Button>
          </div>
          <div className="mt-5 grid gap-3">
            {loadingCompensationProfiles ? (
              <p className="text-base leading-relaxed text-ink-secondary">
                Loading compensation profiles...
              </p>
            ) : null}
            {compensationProfiles.map((profile) => (
              <div
                className="grid gap-3 rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border md:grid-cols-[1fr_auto]"
                key={profile.id}
              >
                <div>
                  <p className="font-mono text-sm font-semibold text-ink-primary">
                    {formatProfile(profile)}
                  </p>
                  <p className="mt-1 text-sm font-semibold text-ink-muted">
                    {profile.cadence}
                  </p>
                </div>
                <Badge variant={statusVariant(profile.metadata.status)}>
                  {profile.metadata.status}
                </Badge>
              </div>
            ))}
            {!loadingCompensationProfiles &&
            compensationProfiles.length === 0 ? (
              <div className="rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border">
                <p className="text-base leading-relaxed text-ink-secondary">
                  No compensation profiles returned for this worker.
                </p>
              </div>
            ) : null}
          </div>
        </Card>
      </div>
    </>
  );
}

function CreateWorkerSheet({
  busy,
  employeeWizard,
  onSubmit,
  open,
  setEmployeeWizard,
  setOpen,
  setStep,
  step,
}: {
  busy: boolean;
  employeeWizard: EmployeeWizardForm;
  onSubmit(event: FormEvent<HTMLFormElement>): Promise<void>;
  open: boolean;
  setEmployeeWizard: Dispatch<SetStateAction<EmployeeWizardForm>>;
  setOpen(open: boolean): void;
  setStep(step: WorkerStep): void;
  step: WorkerStep;
}) {
  const currentIndex = workerSteps.findIndex((item) => item.id === step);

  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetContent className="w-[min(760px,calc(100vw-1rem))] max-w-none overflow-y-auto">
        <SheetHeader>
          <SheetTitle>Create worker</SheetTitle>
          <div className="flex flex-wrap gap-2 pt-2">
            {workerSteps.map((item, index) => (
              <button
                className={cn(
                  "rounded-full px-3 py-1 text-sm font-semibold outline outline-[0.5px] outline-surface-border",
                  item.id === step
                    ? "bg-surface-page text-ink-primary"
                    : "text-ink-secondary",
                )}
                key={item.id}
                onClick={() => setStep(item.id)}
                type="button"
              >
                {index + 1}. {item.label}
              </button>
            ))}
          </div>
        </SheetHeader>

        <form className="grid gap-6" onSubmit={onSubmit}>
          {step === "identity" ? (
            <div className="grid gap-4">
              <div className="grid gap-4 sm:grid-cols-2">
                <Field label="First name">
                  <input
                    className={inputClassName()}
                    onChange={(event) =>
                      setEmployeeWizard((draft) => ({
                        ...draft,
                        first_name: event.target.value,
                      }))
                    }
                    value={employeeWizard.first_name}
                  />
                </Field>
                <Field label="Last name">
                  <input
                    className={inputClassName()}
                    onChange={(event) =>
                      setEmployeeWizard((draft) => ({
                        ...draft,
                        last_name: event.target.value,
                      }))
                    }
                    value={employeeWizard.last_name}
                  />
                </Field>
              </div>
              <Field label="Worker identifier">
                <input
                  className={inputClassName()}
                  onChange={(event) =>
                    setEmployeeWizard((draft) => ({
                      ...draft,
                      identifier: event.target.value,
                    }))
                  }
                  value={employeeWizard.identifier}
                />
              </Field>
            </div>
          ) : null}

          {step === "wallet" ? (
            <div className="grid gap-4">
              <Field label="Wallet address">
                <input
                  className={inputClassName()}
                  onChange={(event) =>
                    setEmployeeWizard((draft) => ({
                      ...draft,
                      wallet_address: event.target.value,
                    }))
                  }
                  value={employeeWizard.wallet_address}
                />
              </Field>
              <Card className="bg-surface-page p-4">
                <div className="flex items-center justify-between gap-4">
                  <div>
                    <p className="font-semibold text-ink-primary">
                      Payroll readiness
                    </p>
                    <p className="mt-1 text-sm font-semibold text-ink-muted">
                      {employeeWizard.wallet_address
                        ? truncateAddress(employeeWizard.wallet_address)
                        : "Missing wallet"}
                    </p>
                  </div>
                  <Badge
                    variant={
                      employeeWizard.wallet_address ? "success" : "warning"
                    }
                  >
                    {employeeWizard.wallet_address ? "Ready" : "Blocked"}
                  </Badge>
                </div>
              </Card>
            </div>
          ) : null}

          {step === "compensation" ? (
            <div className="grid gap-4">
              <label className="flex items-center justify-between gap-4 rounded-lg bg-surface-page p-4 text-sm font-semibold text-ink-primary outline outline-[0.5px] outline-surface-border">
                <span>Create an active compensation profile</span>
                <input
                  checked={employeeWizard.create_compensation}
                  className="size-4 accent-brand-primary"
                  onChange={(event) =>
                    setEmployeeWizard((draft) => ({
                      ...draft,
                      create_compensation: event.target.checked,
                    }))
                  }
                  type="checkbox"
                />
              </label>
              <div className="grid gap-4 sm:grid-cols-2">
                <Field label="Amount units">
                  <input
                    className={inputClassName()}
                    disabled={!employeeWizard.create_compensation}
                    onChange={(event) =>
                      setEmployeeWizard((draft) => ({
                        ...draft,
                        amount_units: event.target.value,
                      }))
                    }
                    value={employeeWizard.amount_units}
                  />
                </Field>
                <Field label="Token">
                  <input
                    className={inputClassName()}
                    disabled={!employeeWizard.create_compensation}
                    onChange={(event) =>
                      setEmployeeWizard((draft) => ({
                        ...draft,
                        token_symbol: event.target.value,
                      }))
                    }
                    value={employeeWizard.token_symbol}
                  />
                </Field>
              </div>
              <div className="grid gap-4 sm:grid-cols-3">
                <Field label="Cadence">
                  <select
                    className={selectClassName()}
                    disabled={!employeeWizard.create_compensation}
                    onChange={(event) =>
                      setEmployeeWizard((draft) => ({
                        ...draft,
                        cadence: event.target
                          .value as EmployeeWizardForm["cadence"],
                      }))
                    }
                    value={employeeWizard.cadence}
                  >
                    <option value="weekly">Weekly</option>
                    <option value="biweekly">Biweekly</option>
                    <option value="monthly">Monthly</option>
                    <option value="custom">Custom</option>
                  </select>
                </Field>
                <Field label="Every">
                  <input
                    className={inputClassName()}
                    disabled={
                      !employeeWizard.create_compensation ||
                      employeeWizard.cadence !== "custom"
                    }
                    onChange={(event) =>
                      setEmployeeWizard((draft) => ({
                        ...draft,
                        cadence_every: event.target.value,
                      }))
                    }
                    type="number"
                    value={employeeWizard.cadence_every}
                  />
                </Field>
                <Field label="Unit">
                  <select
                    className={selectClassName()}
                    disabled={
                      !employeeWizard.create_compensation ||
                      employeeWizard.cadence !== "custom"
                    }
                    onChange={(event) =>
                      setEmployeeWizard((draft) => ({
                        ...draft,
                        cadence_unit: event.target
                          .value as EmployeeWizardForm["cadence_unit"],
                      }))
                    }
                    value={employeeWizard.cadence_unit}
                  >
                    <option value="days">Days</option>
                    <option value="weeks">Weeks</option>
                    <option value="months">Months</option>
                  </select>
                </Field>
              </div>
            </div>
          ) : null}

          {step === "review" ? (
            <div className="grid gap-3">
              {[
                [
                  "Worker",
                  `${employeeWizard.first_name} ${employeeWizard.last_name}`,
                ],
                ["Identifier", employeeWizard.identifier],
                [
                  "Wallet",
                  truncateAddress(employeeWizard.wallet_address || null),
                ],
                [
                  "Compensation",
                  employeeWizard.create_compensation
                    ? `${employeeWizard.amount_units} ${employeeWizard.token_symbol} ${employeeWizard.cadence}`
                    : "Skipped",
                ],
              ].map(([label, value]) => (
                <div
                  className="rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border"
                  key={label}
                >
                  <p className="text-sm font-semibold text-ink-muted">
                    {label}
                  </p>
                  <p className="mt-2 break-all font-mono text-sm font-semibold text-ink-primary">
                    {value}
                  </p>
                </div>
              ))}
            </div>
          ) : null}

          <div className="flex flex-wrap items-center justify-between gap-3 border-t-[0.5px] border-surface-border pt-5">
            <Button
              disabled={currentIndex === 0 || busy}
              onClick={() => setStep(workerSteps[currentIndex - 1].id)}
              type="button"
              variant="outline"
            >
              Back
            </Button>
            <Button disabled={busy} type="submit">
              {busy ? (
                <Loader2 aria-hidden="true" className="animate-spin" />
              ) : null}
              {step === "review" ? "Create worker" : "Continue"}
              <ArrowRight aria-hidden="true" />
            </Button>
          </div>
        </form>
      </SheetContent>
    </Sheet>
  );
}

function CompensationSheet({
  busy,
  compensationForm,
  onSubmit,
  open,
  selectedEmployee,
  setCompensationForm,
  setOpen,
}: {
  busy: boolean;
  compensationForm: CompensationForm;
  onSubmit(event: FormEvent<HTMLFormElement>): Promise<void>;
  open: boolean;
  selectedEmployee?: Employee;
  setCompensationForm: Dispatch<SetStateAction<CompensationForm>>;
  setOpen(open: boolean): void;
}) {
  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetContent className="w-[min(620px,calc(100vw-1rem))] max-w-none overflow-y-auto">
        <SheetHeader>
          <SheetTitle>Add compensation profile</SheetTitle>
          <p className="text-base leading-relaxed text-ink-secondary">
            {selectedEmployee
              ? `${selectedEmployee.first_name} ${selectedEmployee.last_name}`
              : "No worker selected"}
          </p>
        </SheetHeader>
        <form className="grid gap-4" onSubmit={onSubmit}>
          <div className="grid gap-4 sm:grid-cols-2">
            <Field label="Amount units">
              <input
                className={inputClassName()}
                onChange={(event) =>
                  setCompensationForm((draft) => ({
                    ...draft,
                    amount_units: event.target.value,
                  }))
                }
                value={compensationForm.amount_units}
              />
            </Field>
            <Field label="Token">
              <input
                className={inputClassName()}
                onChange={(event) =>
                  setCompensationForm((draft) => ({
                    ...draft,
                    token_symbol: event.target.value,
                  }))
                }
                value={compensationForm.token_symbol}
              />
            </Field>
          </div>
          <div className="grid gap-4 sm:grid-cols-3">
            <Field label="Cadence">
              <select
                className={selectClassName()}
                onChange={(event) =>
                  setCompensationForm((draft) => ({
                    ...draft,
                    cadence: event.target.value as CompensationForm["cadence"],
                  }))
                }
                value={compensationForm.cadence}
              >
                <option value="weekly">Weekly</option>
                <option value="biweekly">Biweekly</option>
                <option value="monthly">Monthly</option>
                <option value="custom">Custom</option>
              </select>
            </Field>
            <Field label="Every">
              <input
                className={inputClassName()}
                disabled={compensationForm.cadence !== "custom"}
                onChange={(event) =>
                  setCompensationForm((draft) => ({
                    ...draft,
                    cadence_every: event.target.value,
                  }))
                }
                type="number"
                value={compensationForm.cadence_every}
              />
            </Field>
            <Field label="Unit">
              <select
                className={selectClassName()}
                disabled={compensationForm.cadence !== "custom"}
                onChange={(event) =>
                  setCompensationForm((draft) => ({
                    ...draft,
                    cadence_unit: event.target
                      .value as CompensationForm["cadence_unit"],
                  }))
                }
                value={compensationForm.cadence_unit}
              >
                <option value="days">Days</option>
                <option value="weeks">Weeks</option>
                <option value="months">Months</option>
              </select>
            </Field>
          </div>
          <Button disabled={busy || !selectedEmployee} type="submit">
            {busy ? (
              <Loader2 aria-hidden="true" className="animate-spin" />
            ) : (
              <Plus aria-hidden="true" />
            )}
            Create profile
          </Button>
        </form>
      </SheetContent>
    </Sheet>
  );
}

function TreasuryView({
  busy,
  deactivateTreasuryAccount,
  selectedTreasuryAccount,
  setSelectedTreasuryAccountId,
  setTreasuryForm,
  setTreasuryDetailOpen,
  setTreasuryDrawerOpen,
  submitTreasury,
  treasuryAccounts,
  treasuryDetailOpen,
  treasuryDrawerOpen,
  treasuryForm,
}: {
  busy: boolean;
  deactivateTreasuryAccount(id: string): Promise<void>;
  selectedTreasuryAccount?: TreasuryAccount;
  setSelectedTreasuryAccountId(value: string): void;
  setTreasuryForm: Dispatch<SetStateAction<TreasuryForm>>;
  setTreasuryDetailOpen(open: boolean): void;
  setTreasuryDrawerOpen(open: boolean): void;
  submitTreasury(event: FormEvent<HTMLFormElement>): Promise<void>;
  treasuryAccounts: TreasuryAccount[];
  treasuryDetailOpen: boolean;
  treasuryDrawerOpen: boolean;
  treasuryForm: TreasuryForm;
}) {
  const activeAccounts = treasuryAccounts.filter(
    (account) => account.metadata.status.toLowerCase() === "active",
  );
  const defaultAccount = treasuryAccounts.find((account) => account.is_default);
  const signatureRequired = treasuryAccounts.filter(
    (account) => account.control_mode === "user_signature_required",
  ).length;

  const openTreasuryAccount = (account: TreasuryAccount) => {
    setSelectedTreasuryAccountId(account.id);
    setTreasuryDetailOpen(true);
  };

  const sheets = (
    <CreateTreasurySheet
      busy={busy}
      onSubmit={submitTreasury}
      open={treasuryDrawerOpen}
      setOpen={setTreasuryDrawerOpen}
      setTreasuryForm={setTreasuryForm}
      treasuryForm={treasuryForm}
    />
  );

  if (treasuryDetailOpen && selectedTreasuryAccount) {
    return (
      <div className="grid gap-6">
        <TreasuryDetailView
          account={selectedTreasuryAccount}
          busy={busy}
          deactivateTreasuryAccount={deactivateTreasuryAccount}
          setTreasuryDetailOpen={setTreasuryDetailOpen}
        />
        {sheets}
      </div>
    );
  }

  return (
    <div className="grid gap-6">
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        {[
          {
            label: "Total accounts",
            value: treasuryAccounts.length,
            detail: "Treasury accounts loaded",
          },
          {
            label: "Active",
            value: activeAccounts.length,
            detail: "Available for payout matching",
          },
          {
            label: "Default treasury",
            value: defaultAccount?.token_symbol ?? "Missing",
            detail: defaultAccount?.name ?? "Create one before payroll",
          },
          {
            label: "Signer review",
            value: signatureRequired,
            detail: "User-signature control mode",
          },
        ].map((metric) => (
          <Card className="p-5" key={metric.label}>
            <p className="text-sm font-semibold text-ink-muted">
              {metric.label}
            </p>
            <p className="mt-4 break-words font-mono text-xl font-semibold text-ink-primary">
              {metric.value}
            </p>
            <p className="mt-2 text-sm font-semibold text-ink-muted">
              {metric.detail}
            </p>
          </Card>
        ))}
      </div>

      <Card className="overflow-hidden">
        <div className="flex flex-col gap-4 border-b-[0.5px] border-surface-border p-5 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <h2 className="text-xl font-semibold tracking-tight">
              Treasury accounts
            </h2>
          </div>
          <Button onClick={() => setTreasuryDrawerOpen(true)} type="button">
            <Plus aria-hidden="true" />
            Add treasury
          </Button>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full min-w-[980px] text-left">
            <thead>
              <tr className="border-b-[0.5px] border-surface-border text-sm font-semibold text-ink-secondary">
                <th className="px-5 py-4">Account</th>
                <th className="px-5 py-4">Token</th>
                <th className="px-5 py-4">Sender</th>
                <th className="px-5 py-4">Custody</th>
                <th className="px-5 py-4">Control mode</th>
                <th className="px-5 py-4">Status</th>
                <th className="px-5 py-4">Actions</th>
              </tr>
            </thead>
            <tbody>
              {treasuryAccounts.map((account) => (
                <tr
                  className="border-b-[0.5px] border-surface-border last:border-b-0 hover:bg-surface-page"
                  key={account.id}
                >
                  <td className="px-5 py-4">
                    <button
                      className="grid gap-1 text-left hover:text-brand-primary"
                      onClick={() => openTreasuryAccount(account)}
                      type="button"
                    >
                      <span className="font-semibold text-ink-primary">
                        {account.name}
                      </span>
                      <span className="text-sm font-semibold text-ink-muted">
                        View treasury details
                      </span>
                    </button>
                  </td>
                  <td className="px-5 py-4">
                    <p className="font-mono text-sm font-semibold text-ink-primary">
                      {account.token_symbol}
                    </p>
                    <p className="mt-1 font-mono text-sm font-semibold text-ink-muted">
                      {account.chain}
                    </p>
                  </td>
                  <td className="px-5 py-4 font-mono text-sm font-semibold text-ink-muted">
                    {truncateAddress(account.sender_address)}
                  </td>
                  <td className="px-5 py-4">
                    <Badge variant="outline">{account.custody_provider}</Badge>
                  </td>
                  <td className="px-5 py-4">
                    <Badge variant="outline">
                      {statusLabel(account.control_mode)}
                    </Badge>
                  </td>
                  <td className="px-5 py-4">
                    <div className="flex flex-wrap gap-2">
                      <Badge variant={statusVariant(account.metadata.status)}>
                        {account.metadata.status}
                      </Badge>
                      {account.is_default ? (
                        <Badge variant="success">Default</Badge>
                      ) : null}
                    </div>
                  </td>
                  <td className="px-5 py-4">
                    <div className="flex flex-wrap gap-2">
                      <Button
                        onClick={() => openTreasuryAccount(account)}
                        size="sm"
                        type="button"
                        variant="secondary"
                      >
                        <Eye aria-hidden="true" />
                        Details
                      </Button>
                      <Button
                        disabled={busy}
                        onClick={() =>
                          void deactivateTreasuryAccount(account.id)
                        }
                        size="sm"
                        type="button"
                        variant="outline"
                      >
                        Deactivate
                      </Button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {treasuryAccounts.length === 0 ? (
            <p className="p-5 text-base leading-relaxed text-ink-secondary">
              No treasury accounts yet.
            </p>
          ) : null}
        </div>
      </Card>

      {sheets}
    </div>
  );
}

function TreasuryDetailView({
  account,
  busy,
  deactivateTreasuryAccount,
  setTreasuryDetailOpen,
}: {
  account: TreasuryAccount;
  busy: boolean;
  deactivateTreasuryAccount(id: string): Promise<void>;
  setTreasuryDetailOpen(open: boolean): void;
}) {
  return (
    <>
      <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
        <Button
          onClick={() => setTreasuryDetailOpen(false)}
          type="button"
          variant="ghost"
        >
          <ChevronLeft aria-hidden="true" />
          Treasury
        </Button>
        <Button
          disabled={busy}
          onClick={() => void deactivateTreasuryAccount(account.id)}
          type="button"
          variant="outline"
        >
          Deactivate account
        </Button>
      </div>

      <Card className="p-6">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <h2 className="text-3xl font-semibold tracking-tight">
              {account.name}
            </h2>
            <p className="mt-2 font-mono text-sm font-semibold text-ink-muted">
              {account.id}
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Badge variant={statusVariant(account.metadata.status)}>
              {account.metadata.status}
            </Badge>
            {account.is_default ? (
              <Badge variant="success">Default</Badge>
            ) : null}
          </div>
        </div>

        <div className="mt-6 grid gap-4 md:grid-cols-3">
          {[
            {
              label: "Token",
              value: account.token_symbol,
            },
            {
              label: "Custody",
              value: statusLabel(account.custody_provider),
            },
            {
              label: "Control mode",
              value: statusLabel(account.control_mode),
            },
          ].map((metric) => (
            <div
              className="rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border"
              key={metric.label}
            >
              <p className="text-sm font-semibold text-ink-muted">
                {metric.label}
              </p>
              <p className="mt-3 break-words font-mono text-lg font-semibold text-ink-primary">
                {metric.value}
              </p>
            </div>
          ))}
        </div>
      </Card>

      <Card className="p-5">
        <h2 className="text-xl font-semibold tracking-tight">
          Account details
        </h2>
        <div className="mt-5 grid gap-3 md:grid-cols-2">
          {[
            ["Account ID", account.id],
            ["Chain", account.chain],
            ["Token address", account.token_address],
            ["Sender address", account.sender_address],
            ["Provider wallet", account.provider_wallet_id ?? "Not configured"],
            ["Provider owner", account.provider_owner_id ?? "Not configured"],
            ["Secret reference", account.secret_reference ?? "Not configured"],
            ["Default", account.is_default ? "Yes" : "No"],
          ].map(([label, value]) => (
            <div
              className="rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border"
              key={label}
            >
              <p className="text-sm font-semibold text-ink-muted">{label}</p>
              <p className="mt-2 break-all font-mono text-sm font-semibold text-ink-primary">
                {value}
              </p>
            </div>
          ))}
        </div>
      </Card>
    </>
  );
}

function CreateTreasurySheet({
  busy,
  onSubmit,
  open,
  setOpen,
  setTreasuryForm,
  treasuryForm,
}: {
  busy: boolean;
  onSubmit(event: FormEvent<HTMLFormElement>): Promise<void>;
  open: boolean;
  setOpen(open: boolean): void;
  setTreasuryForm: Dispatch<SetStateAction<TreasuryForm>>;
  treasuryForm: TreasuryForm;
}) {
  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetContent className="w-[min(680px,calc(100vw-1rem))] max-w-none overflow-y-auto">
        <SheetHeader>
          <SheetTitle>Create treasury</SheetTitle>
          <p className="text-base leading-relaxed text-ink-secondary">
            Add a token treasury account for payroll preview matching.
          </p>
        </SheetHeader>
        <form className="grid gap-4" onSubmit={onSubmit}>
          <Field label="Name">
            <input
              className={inputClassName()}
              onChange={(event) =>
                setTreasuryForm((draft) => ({
                  ...draft,
                  name: event.target.value,
                }))
              }
              value={treasuryForm.name}
            />
          </Field>
          <div className="grid gap-4 sm:grid-cols-2">
            <Field label="Token">
              <input
                className={inputClassName()}
                onChange={(event) =>
                  setTreasuryForm((draft) => ({
                    ...draft,
                    token_symbol: event.target.value,
                  }))
                }
                value={treasuryForm.token_symbol}
              />
            </Field>
            <Field label="Decimals">
              <input
                className={inputClassName()}
                onChange={(event) =>
                  setTreasuryForm((draft) => ({
                    ...draft,
                    token_decimals: Number(event.target.value),
                  }))
                }
                type="number"
                value={treasuryForm.token_decimals}
              />
            </Field>
          </div>
          <Field label="Token address">
            <input
              className={inputClassName()}
              onChange={(event) =>
                setTreasuryForm((draft) => ({
                  ...draft,
                  token_address: event.target.value,
                }))
              }
              value={treasuryForm.token_address}
            />
          </Field>
          <Field label="Sender address">
            <input
              className={inputClassName()}
              onChange={(event) =>
                setTreasuryForm((draft) => ({
                  ...draft,
                  sender_address: event.target.value,
                }))
              }
              value={treasuryForm.sender_address}
            />
          </Field>
          <div className="grid gap-4 sm:grid-cols-2">
            <Field label="Custody provider">
              <select
                className={selectClassName()}
                onChange={(event) =>
                  setTreasuryForm((draft) => ({
                    ...draft,
                    custody_provider: event.target
                      .value as TreasuryForm["custody_provider"],
                  }))
                }
                value={treasuryForm.custody_provider}
              >
                <option value="local_key">Local key</option>
                <option value="privy">Privy</option>
                <option value="external">External</option>
              </select>
            </Field>
            <Field label="Control mode">
              <select
                className={selectClassName()}
                onChange={(event) =>
                  setTreasuryForm((draft) => ({
                    ...draft,
                    control_mode: event.target
                      .value as TreasuryForm["control_mode"],
                  }))
                }
                value={treasuryForm.control_mode}
              >
                <option value="server_controlled">Server controlled</option>
                <option value="user_delegated">User delegated</option>
                <option value="user_signature_required">
                  Signature required
                </option>
                <option value="external_execution">External execution</option>
              </select>
            </Field>
          </div>
          <Field label="Secret reference">
            <input
              className={inputClassName()}
              onChange={(event) =>
                setTreasuryForm((draft) => ({
                  ...draft,
                  secret_reference: event.target.value,
                }))
              }
              value={treasuryForm.secret_reference}
            />
          </Field>
          <label className="flex items-center justify-between gap-4 rounded-lg bg-surface-page p-4 text-sm font-semibold text-ink-primary outline outline-[0.5px] outline-surface-border">
            <span>Set as default account</span>
            <input
              checked={treasuryForm.is_default}
              className="size-4 accent-brand-primary"
              onChange={(event) =>
                setTreasuryForm((draft) => ({
                  ...draft,
                  is_default: event.target.checked,
                }))
              }
              type="checkbox"
            />
          </label>
          <Button disabled={busy} type="submit">
            {busy ? (
              <Loader2 aria-hidden="true" className="animate-spin" />
            ) : (
              <Plus aria-hidden="true" />
            )}
            Create treasury
          </Button>
        </form>
      </SheetContent>
    </Sheet>
  );
}

function PayrollWorkspace({
  createPayrollRun,
  executeReadyPayroll,
  execution,
  payrollDraft,
  payrollBusy,
  payrollMode,
  payrollRuns,
  payrollTab,
  pollingExecution,
  preview,
  refreshPreview,
  setPayrollDraft,
  setPayrollMode,
  setPayrollTab,
}: {
  createPayrollRun(): void;
  executeReadyPayroll(): Promise<void>;
  execution: PayrollExecution | null;
  payrollDraft: PayrollDraft;
  payrollBusy: boolean;
  payrollMode: PayrollMode;
  payrollRuns: PayrollRunRecord[];
  payrollTab: PayrollTab;
  pollingExecution: boolean;
  preview: PayrollPreview | null;
  refreshPreview(nextStep?: PayrollTab): Promise<void>;
  setPayrollDraft: Dispatch<SetStateAction<PayrollDraft>>;
  setPayrollMode(mode: PayrollMode): void;
  setPayrollTab(tab: PayrollTab): void;
}) {
  const readyItems = useMemo(
    () => selectedReadyItems(preview, payrollDraft.selectedEmployeeIds),
    [payrollDraft.selectedEmployeeIds, preview],
  );
  const allReadyItems = useMemo(() => previewReadyItems(preview), [preview]);
  const attemptsByEmployee = useMemo(() => {
    const map = new Map<string, PayrollExecutionAttempt>();
    execution?.attempts.forEach((attempt) => {
      map.set(attempt.employeeId, attempt);
    });
    return map;
  }, [execution]);

  const executionCounts = useMemo(() => {
    const attempts = execution?.attempts ?? [];
    return {
      completed: attempts.filter((attempt) => attempt.status === "completed")
        .length,
      failed: attempts.filter((attempt) => attempt.status === "failed").length,
      inTransit: attempts.filter((attempt) =>
        ["queued", "submitting", "submitted"].includes(attempt.status),
      ).length,
    };
  }, [execution]);

  const executionRunning = execution
    ? !isExecutionTerminal(execution.status)
    : false;
  const action =
    payrollTab === "preview"
      ? preview
        ? "Continue to review"
        : "Preview payroll"
      : payrollTab === "review"
        ? readyItems.length > 0
          ? "Continue to confirmation"
          : allReadyItems.length > 0
            ? "Select payments to continue"
            : "Refresh preview"
        : payrollTab === "confirm"
          ? "Start execution"
          : executionRunning
            ? "Execution in progress"
            : "Run new preview";
  const primaryDisabled =
    payrollBusy ||
    executionRunning ||
    (payrollTab === "review" && readyItems.length === 0) ||
    (payrollTab === "confirm" && readyItems.length === 0);

  function runPrimaryAction() {
    if (payrollTab === "preview") {
      if (!preview) {
        void refreshPreview();
        return;
      }

      setPayrollTab("review");
      return;
    }

    if (payrollTab === "review") {
      if (!preview || allReadyItems.length === 0) {
        void refreshPreview();
        return;
      }

      if (readyItems.length === 0) {
        return;
      }

      setPayrollTab("confirm");
      return;
    }

    if (payrollTab === "confirm") {
      if (readyItems.length > 0) {
        void executeReadyPayroll();
      }
      return;
    }

    if (executionRunning) {
      return;
    }

    if (payrollTab === "execution") {
      void refreshPreview("review");
    }
  }

  if (payrollMode === "list") {
    return (
      <PayrollRunsView
        createPayrollRun={createPayrollRun}
        payrollBusy={payrollBusy}
        payrollRuns={payrollRuns}
      />
    );
  }

  return (
    <div className="grid gap-6" data-testid="payroll-workspace">
      <div>
        <Button
          onClick={() => setPayrollMode("list")}
          type="button"
          variant="ghost"
        >
          <ChevronLeft aria-hidden="true" />
          Payroll runs
        </Button>
      </div>

      <Card className="p-5">
        <PayrollWizardStepper
          execution={execution}
          payrollTab={payrollTab}
          preview={preview}
          setPayrollTab={setPayrollTab}
        />
      </Card>

      <Card className="overflow-hidden">
        {payrollTab === "preview" ? (
          <PayrollPreviewTable
            attemptsByEmployee={attemptsByEmployee}
            preview={preview}
          />
        ) : null}

        {payrollTab === "review" ? (
          <PayrollReviewStep
            attemptsByEmployee={attemptsByEmployee}
            payrollDraft={payrollDraft}
            preview={preview}
            refreshPreview={() => void refreshPreview("review")}
            setPayrollDraft={setPayrollDraft}
          />
        ) : null}

        {payrollTab === "confirm" ? (
          <PayrollConfirmStep
            executeReadyPayroll={executeReadyPayroll}
            payrollDraft={payrollDraft}
            payrollBusy={payrollBusy}
            preview={preview}
            setPayrollTab={setPayrollTab}
          />
        ) : null}

        {payrollTab === "execution" ? (
          <PayrollExecutionPanel
            execution={execution}
            executionCounts={executionCounts}
            payrollBusy={payrollBusy}
            pollingExecution={pollingExecution}
            preview={preview}
          />
        ) : null}
      </Card>

      <Card className="p-4">
        <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <p className="font-mono text-sm font-semibold text-ink-muted">
            {preview
              ? `${readyItems.length}/${allReadyItems.length} selected · ${payableDisplayForItems(
                  readyItems,
                )}`
              : "Preview is loading"}
          </p>
          <Button
            disabled={primaryDisabled}
            onClick={runPrimaryAction}
            type="button"
          >
            {payrollBusy ? (
              <Loader2 aria-hidden="true" className="animate-spin" />
            ) : (
              <FileClock aria-hidden="true" />
            )}
            {action}
          </Button>
        </div>
      </Card>
    </div>
  );
}

function PayrollRunsView({
  createPayrollRun,
  payrollBusy,
  payrollRuns,
}: {
  createPayrollRun(): void;
  payrollBusy: boolean;
  payrollRuns: PayrollRunRecord[];
}) {
  const completedRuns = payrollRuns.filter(
    (run) => run.status === "completed",
  ).length;
  const openRuns = payrollRuns.filter(
    (run) => run.status === "draft" || run.status === "review_required",
  ).length;
  const totalReadyPayments = payrollRuns.reduce(
    (sum, run) => sum + run.readyPaymentCount,
    0,
  );

  return (
    <div className="grid gap-6" data-testid="payroll-runs-view">
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        {[
          {
            label: "Payroll runs",
            value: payrollRuns.length,
            detail: "Created runs",
          },
          {
            label: "Completed",
            value: completedRuns,
            detail: "Settled runs",
          },
          {
            label: "Open review",
            value: openRuns,
            detail: "Draft or needs review",
          },
          {
            label: "Ready payments",
            value: totalReadyPayments,
            detail: "Across visible runs",
          },
        ].map((metric) => (
          <Card className="p-5" key={metric.label}>
            <p className="text-sm font-semibold text-ink-muted">
              {metric.label}
            </p>
            <p className="mt-4 break-words font-mono text-xl font-semibold text-ink-primary">
              {metric.value}
            </p>
            <p className="mt-2 text-sm font-semibold text-ink-muted">
              {metric.detail}
            </p>
          </Card>
        ))}
      </div>

      <Card className="overflow-hidden">
        <div className="flex flex-col gap-4 border-b-[0.5px] border-surface-border p-5 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <h2 className="text-xl font-semibold tracking-tight">
              Payroll runs
            </h2>
            <p className="mt-2 text-base leading-relaxed text-ink-secondary">
              Review previous runs first, then create a new run when you are
              ready to preview payroll.
            </p>
          </div>
          <Button
            disabled={payrollBusy}
            onClick={createPayrollRun}
            type="button"
          >
            {payrollBusy ? (
              <Loader2 aria-hidden="true" className="animate-spin" />
            ) : (
              <Plus aria-hidden="true" />
            )}
            New payroll run
          </Button>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full min-w-[940px] text-left">
            <thead>
              <tr className="border-b-[0.5px] border-surface-border text-sm font-semibold text-ink-secondary">
                <th className="px-5 py-4">Run</th>
                <th className="px-5 py-4">Run date</th>
                <th className="px-5 py-4">Total</th>
                <th className="px-5 py-4">Workers</th>
                <th className="px-5 py-4">Ready</th>
                <th className="px-5 py-4">Blocked</th>
                <th className="px-5 py-4">Status</th>
              </tr>
            </thead>
            <tbody>
              {payrollRuns.map((run) => (
                <tr
                  className="border-b-[0.5px] border-surface-border last:border-b-0 hover:bg-surface-page"
                  key={run.id}
                >
                  <td className="px-5 py-4">
                    <p className="font-semibold text-ink-primary">{run.name}</p>
                    <p className="mt-1 font-mono text-sm font-semibold text-ink-muted">
                      {run.id}
                    </p>
                  </td>
                  <td className="px-5 py-4 font-mono text-sm font-semibold text-ink-primary">
                    {run.runDate}
                  </td>
                  <td className="px-5 py-4 font-mono text-sm font-semibold text-ink-primary">
                    {run.totalDisplay}
                  </td>
                  <td className="px-5 py-4 font-mono text-sm font-semibold text-ink-primary">
                    {run.employeeCount}
                  </td>
                  <td className="px-5 py-4">
                    <Badge variant="success">{run.readyPaymentCount}</Badge>
                  </td>
                  <td className="px-5 py-4">
                    <Badge
                      variant={run.blockedCount > 0 ? "warning" : "outline"}
                    >
                      {run.blockedCount}
                    </Badge>
                  </td>
                  <td className="px-5 py-4">
                    <Badge variant={payrollRunVariant(run.status)}>
                      {statusLabel(run.status)}
                    </Badge>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {payrollRuns.length === 0 ? (
            <p className="p-5 text-base leading-relaxed text-ink-secondary">
              No payroll runs yet.
            </p>
          ) : null}
        </div>
      </Card>
    </div>
  );
}

function PayrollWizardStepper({
  execution,
  payrollTab,
  preview,
  setPayrollTab,
}: {
  execution: PayrollExecution | null;
  payrollTab: PayrollTab;
  preview: PayrollPreview | null;
  setPayrollTab(tab: PayrollTab): void;
}) {
  const reducedMotion = useReducedMotion();
  const currentIndex = payrollWizardSteps.findIndex(
    (step) => step.id === payrollTab,
  );
  const readyCount = previewReadyItems(preview).length;

  function canOpenStep(step: PayrollTab) {
    if (step === "preview") {
      return true;
    }

    if (step === "review") {
      return Boolean(preview);
    }

    if (step === "confirm") {
      return readyCount > 0;
    }

    return Boolean(execution);
  }

  function isStepComplete(step: PayrollTab, index: number) {
    if (step === "preview") {
      return Boolean(preview) && currentIndex > index;
    }

    if (step === "review") {
      return currentIndex > index || Boolean(execution);
    }

    if (step === "confirm") {
      return Boolean(execution);
    }

    return execution ? isExecutionTerminal(execution.status) : false;
  }

  return (
    <div className="grid gap-3 lg:grid-cols-4">
      {payrollWizardSteps.map((step, index) => {
        const isCurrent = payrollTab === step.id;
        const complete = isStepComplete(step.id, index);
        const disabled = !canOpenStep(step.id);

        return (
          <button
            className={cn(
              "rounded-lg bg-surface-page p-4 text-left outline outline-[0.5px] outline-surface-border transition-colors",
              isCurrent && "bg-surface-card outline-ink-muted",
              disabled
                ? "cursor-not-allowed opacity-60"
                : "hover:bg-surface-card",
            )}
            disabled={disabled}
            key={step.id}
            onClick={() => setPayrollTab(step.id)}
            type="button"
          >
            <div className="flex items-center gap-3">
              <motion.span
                animate={
                  isCurrent && !reducedMotion
                    ? { scale: [1, 1.08, 1] }
                    : { scale: 1 }
                }
                className={cn(
                  "inline-flex size-8 items-center justify-center rounded-full outline outline-[0.5px] outline-surface-border",
                  complete
                    ? "bg-success-bg text-success"
                    : isCurrent
                      ? "bg-surface-card text-ink-primary"
                      : "text-ink-muted",
                )}
                transition={{
                  duration: 1.2,
                  repeat:
                    isCurrent && !reducedMotion ? Number.POSITIVE_INFINITY : 0,
                }}
              >
                {complete ? (
                  <CheckCircle2 aria-hidden="true" className="size-4" />
                ) : (
                  <Circle aria-hidden="true" className="size-4" />
                )}
              </motion.span>
              <div>
                <p className="font-semibold text-ink-primary">
                  {index + 1}. {step.label}
                </p>
                <p className="mt-1 text-sm font-semibold text-ink-muted">
                  {step.description}
                </p>
              </div>
            </div>
          </button>
        );
      })}
    </div>
  );
}

function PayrollPreviewTable({
  attemptsByEmployee,
  preview,
}: {
  attemptsByEmployee: Map<string, PayrollExecutionAttempt>;
  preview: PayrollPreview | null;
}) {
  if (!preview) {
    return (
      <div className="p-5">
        <Badge variant="warning">Preparing preview</Badge>
      </div>
    );
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full min-w-[1020px] text-left">
        <thead>
          <tr className="border-b-[0.5px] border-surface-border text-sm font-semibold text-ink-secondary">
            <th className="px-5 py-4">Worker</th>
            <th className="px-5 py-4">Wallet</th>
            <th className="px-5 py-4">Compensation</th>
            <th className="px-5 py-4">Treasury</th>
            <th className="px-5 py-4">Blockers</th>
            <th className="px-5 py-4">Execution</th>
            <th className="px-5 py-4">Explorer</th>
          </tr>
        </thead>
        <tbody>
          {preview.items.map((item) => {
            const attempt = attemptsByEmployee.get(item.employeeId);
            return (
              <tr
                className="border-b-[0.5px] border-surface-border last:border-b-0"
                key={item.employeeId}
              >
                <td className="px-5 py-4">
                  <p className="font-semibold text-ink-primary">
                    {item.workerName}
                  </p>
                  <p className="mt-1 font-mono text-sm font-semibold text-ink-muted">
                    {item.identifier}
                  </p>
                </td>
                <td className="px-5 py-4">
                  <Badge variant={item.walletAddress ? "success" : "warning"}>
                    {truncateAddress(item.walletAddress)}
                  </Badge>
                </td>
                <td className="px-5 py-4">
                  <p className="font-mono text-sm font-semibold text-ink-primary">
                    {item.amount?.display ?? "Missing"}
                  </p>
                  <p className="mt-1 text-sm font-semibold text-ink-muted">
                    {item.cadenceLabel ?? item.cadence}
                  </p>
                </td>
                <td className="px-5 py-4">
                  <p className="font-mono text-sm font-semibold text-ink-primary">
                    {item.treasury?.tokenSymbol ?? "Missing"}
                  </p>
                  <p className="mt-1 text-sm font-semibold text-ink-muted">
                    {item.treasury?.controlMode ?? "No account"}
                  </p>
                </td>
                <td className="px-5 py-4">
                  <Badge variant={itemVariant(item)}>
                    {item.blockers.length === 0
                      ? "Payable"
                      : `${item.blockers.length} blocker`}
                  </Badge>
                </td>
                <td className="px-5 py-4">
                  {attempt ? (
                    <Badge variant={attemptVariant(attempt.status)}>
                      {attempt.label}
                    </Badge>
                  ) : (
                    <Badge variant="outline">Not submitted</Badge>
                  )}
                </td>
                <td className="px-5 py-4">
                  {attempt?.transactionHash ? (
                    <a
                      className="inline-flex items-center gap-2 rounded-full px-3 py-2 text-sm font-semibold text-ink-primary outline outline-[0.5px] outline-surface-border hover:bg-surface-page"
                      href={explorerUrl(attempt.transactionHash)}
                      rel="noreferrer"
                      target="_blank"
                    >
                      View tx
                      <ExternalLink aria-hidden="true" className="size-4" />
                    </a>
                  ) : (
                    <span className="text-sm font-semibold text-ink-muted">
                      Pending
                    </span>
                  )}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

function PayrollReviewStep({
  attemptsByEmployee,
  payrollDraft,
  preview,
  refreshPreview,
  setPayrollDraft,
}: {
  attemptsByEmployee: Map<string, PayrollExecutionAttempt>;
  payrollDraft: PayrollDraft;
  preview: PayrollPreview | null;
  refreshPreview(): void;
  setPayrollDraft: Dispatch<SetStateAction<PayrollDraft>>;
}) {
  const ready = previewReadyItems(preview);
  const selected = selectedReadyItems(
    preview,
    payrollDraft.selectedEmployeeIds,
  );
  const blocked = previewBlockedItems(preview);

  if (!preview) {
    return (
      <div className="p-5">
        <Badge variant="warning">Preparing review</Badge>
      </div>
    );
  }

  const toggleEmployee = (employeeId: string) => {
    setPayrollDraft((current) => {
      const selectedIds = new Set(current.selectedEmployeeIds);

      if (selectedIds.has(employeeId)) {
        selectedIds.delete(employeeId);
      } else {
        selectedIds.add(employeeId);
      }

      return {
        ...current,
        selectedEmployeeIds: Array.from(selectedIds),
      };
    });
  };

  return (
    <div className="grid">
      <div className="grid gap-4 border-b-[0.5px] border-surface-border p-5">
        <div className="grid gap-4 lg:grid-cols-[1fr_auto] lg:items-center">
          <div>
            <h3 className="text-xl font-semibold tracking-tight">
              Review payments
            </h3>
            <p className="mt-2 text-base leading-relaxed text-ink-secondary">
              Set the run details and choose the ready workers to include.
            </p>
          </div>
          <Button onClick={refreshPreview} type="button" variant="secondary">
            <RefreshCw aria-hidden="true" />
            Refresh preview
          </Button>
        </div>
        <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_180px_minmax(220px,0.8fr)]">
          <Field label="Run name">
            <input
              className={inputClassName()}
              onChange={(event) =>
                setPayrollDraft((current) => ({
                  ...current,
                  name: event.target.value,
                }))
              }
              value={payrollDraft.name}
            />
          </Field>
          <Field label="Run date">
            <input
              className={inputClassName()}
              onChange={(event) =>
                setPayrollDraft((current) => ({
                  ...current,
                  runDate: event.target.value,
                }))
              }
              type="date"
              value={payrollDraft.runDate}
            />
          </Field>
          <Field label="Internal note">
            <input
              className={inputClassName()}
              onChange={(event) =>
                setPayrollDraft((current) => ({
                  ...current,
                  note: event.target.value,
                }))
              }
              placeholder="Optional context for approvers"
              value={payrollDraft.note}
            />
          </Field>
        </div>
        <div className="flex flex-wrap gap-2">
          <Badge variant={selected.length > 0 ? "success" : "warning"}>
            {selected.length}/{ready.length} selected
          </Badge>
          <Badge variant={blocked.length > 0 ? "warning" : "success"}>
            {blocked.length} needs review
          </Badge>
          <Badge variant="outline">{payableDisplayForItems(selected)}</Badge>
        </div>
      </div>

      <div className="overflow-x-auto">
        <table className="w-full min-w-[880px] text-left">
          <thead>
            <tr className="border-b-[0.5px] border-surface-border text-sm font-semibold text-ink-secondary">
              <th className="px-4 py-3">Include</th>
              <th className="px-4 py-3">Worker</th>
              <th className="px-4 py-3">Payment</th>
              <th className="px-4 py-3">Status</th>
              <th className="px-4 py-3">Next action</th>
            </tr>
          </thead>
          <tbody>
            {preview.items.map((item) => {
              const attempt = attemptsByEmployee.get(item.employeeId);
              const primaryBlocker = item.blockers[0];
              const isReady = item.blockers.length === 0;
              const isSelected = payrollDraft.selectedEmployeeIds.includes(
                item.employeeId,
              );

              return (
                <tr
                  className="border-b-[0.5px] border-surface-border last:border-b-0"
                  key={item.employeeId}
                >
                  <td className="px-4 py-3">
                    <input
                      aria-label={`Include ${item.workerName}`}
                      checked={isSelected}
                      className="size-4 accent-brand-primary disabled:opacity-50"
                      disabled={!isReady}
                      onChange={() => toggleEmployee(item.employeeId)}
                      type="checkbox"
                    />
                  </td>
                  <td className="px-4 py-3">
                    <p className="font-semibold text-ink-primary">
                      {item.workerName}
                    </p>
                    <p className="mt-1 font-mono text-sm font-semibold text-ink-muted">
                      {item.identifier}
                    </p>
                    <Badge
                      className="mt-3"
                      variant={item.walletAddress ? "success" : "warning"}
                    >
                      {truncateAddress(item.walletAddress)}
                    </Badge>
                  </td>
                  <td className="px-4 py-3">
                    <p className="font-mono text-sm font-semibold text-ink-primary">
                      {item.amount?.display ?? "Missing"}
                    </p>
                    <p className="mt-1 text-sm font-semibold text-ink-muted">
                      {item.treasury
                        ? `${item.treasury.tokenSymbol} · ${statusLabel(
                            item.treasury.controlMode,
                          )}`
                        : "No treasury account"}
                    </p>
                  </td>
                  <td className="px-4 py-3">
                    <Badge variant={itemVariant(item)}>
                      {item.blockers.length === 0
                        ? "Ready"
                        : `${item.blockers.length} blocker`}
                    </Badge>
                  </td>
                  <td className="px-4 py-3">
                    <p className="max-w-[320px] text-base leading-relaxed text-ink-secondary">
                      {attempt
                        ? attempt.label
                        : primaryBlocker
                          ? `${formatBlocker(primaryBlocker)}: ${blockerResolution(
                              primaryBlocker,
                            )}`
                          : isSelected
                            ? "Included in the confirmation step."
                            : "Excluded from this run."}
                    </p>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function PayrollConfirmStep({
  executeReadyPayroll,
  payrollDraft,
  payrollBusy,
  preview,
  setPayrollTab,
}: {
  executeReadyPayroll(): Promise<void>;
  payrollDraft: PayrollDraft;
  payrollBusy: boolean;
  preview: PayrollPreview | null;
  setPayrollTab(tab: PayrollTab): void;
}) {
  const ready = selectedReadyItems(preview, payrollDraft.selectedEmployeeIds);
  const blocked = previewBlockedItems(preview);

  if (!preview) {
    return (
      <div className="p-5">
        <Badge variant="warning">Preview required</Badge>
      </div>
    );
  }

  return (
    <div className="grid gap-5 p-5">
      <div className="rounded-lg bg-surface-page p-5 outline outline-[0.5px] outline-surface-border">
        <div className="grid gap-5 lg:grid-cols-[1fr_auto] lg:items-center">
          <div>
            <Badge variant={ready.length > 0 ? "success" : "warning"}>
              {ready.length} ready payment{ready.length === 1 ? "" : "s"}
            </Badge>
            <h3 className="mt-4 text-2xl font-semibold tracking-tight">
              {payrollDraft.name}
            </h3>
            <p className="mt-2 text-base leading-relaxed text-ink-secondary">
              Blocked workers stay out of this mocked execution. The backend
              preview remains the source of truth for readiness.
            </p>
          </div>
          <div className="flex flex-wrap gap-3">
            <Button
              onClick={() => setPayrollTab("review")}
              type="button"
              variant="secondary"
            >
              Review again
            </Button>
            <Button
              disabled={payrollBusy || ready.length === 0}
              onClick={() => void executeReadyPayroll()}
              type="button"
            >
              {payrollBusy ? (
                <Loader2 aria-hidden="true" className="animate-spin" />
              ) : (
                <ArrowRight aria-hidden="true" />
              )}
              Start execution
            </Button>
          </div>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-3">
        {[
          ["Execution total", payableDisplayForItems(ready)],
          ["Ready workers", ready.length],
          ["Run date", payrollDraft.runDate],
        ].map(([label, value]) => (
          <div
            className="rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border"
            key={label}
          >
            <p className="text-sm font-semibold text-ink-muted">{label}</p>
            <p className="mt-3 break-words font-mono text-lg font-semibold text-ink-primary">
              {value}
            </p>
          </div>
        ))}
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <div className="rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border">
          <p className="text-sm font-semibold text-ink-muted">
            Excluded blockers
          </p>
          <p className="mt-3 font-mono text-lg font-semibold text-ink-primary">
            {blocked.length}
          </p>
        </div>
        <div className="rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border">
          <p className="text-sm font-semibold text-ink-muted">Internal note</p>
          <p className="mt-3 text-base leading-relaxed text-ink-secondary">
            {payrollDraft.note || "No note added."}
          </p>
        </div>
      </div>

      <div className="overflow-x-auto rounded-lg outline outline-[0.5px] outline-surface-border">
        <table className="w-full min-w-[720px] text-left">
          <thead>
            <tr className="border-b-[0.5px] border-surface-border text-sm font-semibold text-ink-secondary">
              <th className="px-4 py-3">Worker</th>
              <th className="px-4 py-3">Amount</th>
              <th className="px-4 py-3">Wallet</th>
              <th className="px-4 py-3">Treasury</th>
            </tr>
          </thead>
          <tbody>
            {ready.map((item) => (
              <tr
                className="border-b-[0.5px] border-surface-border last:border-b-0"
                key={item.employeeId}
              >
                <td className="px-4 py-3">
                  <p className="font-semibold text-ink-primary">
                    {item.workerName}
                  </p>
                  <p className="mt-1 font-mono text-sm font-semibold text-ink-muted">
                    {item.identifier}
                  </p>
                </td>
                <td className="px-4 py-3 font-mono text-sm font-semibold text-ink-primary">
                  {item.amount?.display ?? "Missing"}
                </td>
                <td className="px-4 py-3">
                  <Badge variant="success">
                    {truncateAddress(item.walletAddress)}
                  </Badge>
                </td>
                <td className="px-4 py-3">
                  <Badge variant="outline">
                    {item.treasury?.tokenSymbol ?? "Missing"}
                  </Badge>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {ready.length === 0 ? (
          <p className="p-5 text-base leading-relaxed text-ink-secondary">
            No workers are ready for execution.
          </p>
        ) : null}
      </div>
    </div>
  );
}

function PayrollExecutionPanel({
  execution,
  executionCounts,
  payrollBusy,
  pollingExecution,
  preview,
}: {
  execution: PayrollExecution | null;
  executionCounts: { completed: number; failed: number; inTransit: number };
  payrollBusy: boolean;
  pollingExecution: boolean;
  preview: PayrollPreview | null;
}) {
  const reducedMotion = useReducedMotion();
  const readyCount = previewReadyItems(preview).length;
  const progress = executionProgress(execution);
  const executionRunning = execution
    ? !isExecutionTerminal(execution.status)
    : false;

  if (!execution) {
    return (
      <div className="grid gap-4 p-5">
        <div className="rounded-lg bg-surface-page p-5 outline outline-[0.5px] outline-surface-border">
          <Badge variant="outline">Execution not started</Badge>
          <h3 className="mt-4 text-2xl font-semibold tracking-tight">
            Confirm the batch before execution
          </h3>
          <p className="mt-2 text-base leading-relaxed text-ink-secondary">
            {readyCount} ready payment{readyCount === 1 ? "" : "s"} will appear
            here once the mocked execution adapter starts.
          </p>
          {payrollBusy ? (
            <div className="mt-5">
              <ExecutionProgressBar progress={18} />
            </div>
          ) : null}
        </div>
      </div>
    );
  }

  return (
    <div className="grid gap-5 p-5">
      <div className="rounded-lg bg-surface-page p-5 outline outline-[0.5px] outline-surface-border">
        <div className="grid gap-5 lg:grid-cols-[1fr_auto] lg:items-start">
          <div>
            <div className="flex flex-wrap gap-2">
              <Badge variant={executionVariant(execution.status)}>
                {statusLabel(execution.status)}
              </Badge>
              {pollingExecution ? (
                <Badge variant="warning">Auto-refreshing</Badge>
              ) : null}
              <Badge variant="outline">Mock execution</Badge>
            </div>
            <h3 className="mt-4 text-2xl font-semibold tracking-tight">
              {executionRunning
                ? "Submitting payout attempts"
                : "Execution finished"}
            </h3>
            <p className="mt-2 font-mono text-sm font-semibold text-ink-muted">
              {execution.id}
            </p>
          </div>
          <div className="min-w-[220px]">
            <p className="text-sm font-semibold text-ink-muted">
              Overall progress
            </p>
            <p className="mt-2 font-mono text-2xl font-semibold text-ink-primary">
              {progress}%
            </p>
          </div>
        </div>
        <div className="mt-5">
          <ExecutionProgressBar progress={progress} />
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-3">
        {[
          ["Completed", executionCounts.completed, "success"],
          ["In transit", executionCounts.inTransit, "warning"],
          ["Failed", executionCounts.failed, "error"],
        ].map(([label, value, variant]) => (
          <motion.div
            animate={
              !reducedMotion && label === "In transit" && executionRunning
                ? { y: [0, -2, 0] }
                : { y: 0 }
            }
            className="rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border"
            key={label}
            transition={{
              duration: 1.4,
              repeat:
                !reducedMotion && label === "In transit" && executionRunning
                  ? Number.POSITIVE_INFINITY
                  : 0,
            }}
          >
            <div className="flex items-center justify-between">
              <p className="text-sm font-semibold text-ink-muted">{label}</p>
              <Badge variant={variant as "success" | "warning" | "error"}>
                {value}
              </Badge>
            </div>
          </motion.div>
        ))}
      </div>

      <PayrollExecutionRail execution={execution} />

      <div className="overflow-x-auto rounded-lg outline outline-[0.5px] outline-surface-border">
        <table className="w-full min-w-[720px] text-left">
          <thead>
            <tr className="border-b-[0.5px] border-surface-border text-sm font-semibold text-ink-secondary">
              <th className="px-4 py-3">Worker</th>
              <th className="px-4 py-3">Amount</th>
              <th className="px-4 py-3">Status</th>
              <th className="px-4 py-3">Progress</th>
              <th className="px-4 py-3">Transaction</th>
            </tr>
          </thead>
          <tbody>
            {execution.attempts.map((attempt, index) => (
              <motion.tr
                animate={{ opacity: 1, y: 0 }}
                className="border-b-[0.5px] border-surface-border last:border-b-0"
                initial={reducedMotion ? false : { opacity: 0, y: 6 }}
                key={attempt.id}
                transition={{ delay: reducedMotion ? 0 : index * 0.04 }}
              >
                <td className="px-4 py-3 font-semibold text-ink-primary">
                  {attempt.workerName}
                </td>
                <td className="px-4 py-3 font-mono text-sm font-semibold text-ink-primary">
                  {attempt.amount.display}
                </td>
                <td className="px-4 py-3">
                  <Badge variant={attemptVariant(attempt.status)}>
                    {attempt.label}
                  </Badge>
                </td>
                <td className="px-4 py-3">
                  <ExecutionProgressBar
                    progress={attemptProgress(attempt.status)}
                  />
                </td>
                <td className="px-4 py-3">
                  {attempt.transactionHash ? (
                    <a
                      className="inline-flex items-center gap-2 rounded-full px-3 py-2 font-mono text-sm font-semibold text-link underline decoration-link underline-offset-4 outline outline-[0.5px] outline-surface-border hover:bg-surface-card hover:text-link-hover focus-visible:text-link-hover"
                      href={explorerUrl(attempt.transactionHash)}
                      rel="noreferrer"
                      target="_blank"
                    >
                      {truncateAddress(attempt.transactionHash)}
                      <ExternalLink aria-hidden="true" className="size-4" />
                    </a>
                  ) : (
                    <span className="text-sm font-semibold text-ink-muted">
                      Not available
                    </span>
                  )}
                </td>
              </motion.tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function ExecutionProgressBar({ progress }: { progress: number }) {
  const boundedProgress = Math.max(0, Math.min(100, progress));

  return (
    <div className="h-3 overflow-hidden rounded-full bg-surface-card outline outline-[0.5px] outline-surface-border">
      <motion.div
        animate={{ width: `${boundedProgress}%` }}
        className="h-full rounded-full bg-ink-secondary"
        initial={false}
        transition={{
          duration: 0.35,
          ease: "easeOut",
        }}
      />
    </div>
  );
}

function PayrollExecutionRail({ execution }: { execution: PayrollExecution }) {
  const reducedMotion = useReducedMotion();
  const executionRunning = !isExecutionTerminal(execution.status);
  const steps = [
    {
      label: "Preview",
      value: "completed",
      variant: "success",
      active: false,
    },
    {
      label: "Review",
      value: "completed",
      variant: "success",
      active: false,
    },
    {
      label: "Create payrun",
      value: "completed",
      variant: "success",
      active: false,
    },
    {
      label: "Submit payouts",
      value: executionRunning ? execution.status : "completed",
      variant: executionRunning ? "warning" : "success",
      active: executionRunning,
    },
    {
      label: "Settlement",
      value: executionRunning ? "waiting" : execution.status,
      variant: executionRunning
        ? "outline"
        : executionVariant(execution.status),
      active: false,
    },
  ] as const;

  return (
    <div className="rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border">
      <div className="grid gap-3 lg:grid-cols-5">
        {steps.map((step, index) => (
          <motion.div
            animate={
              step.active && !reducedMotion ? { y: [0, -3, 0] } : { y: 0 }
            }
            className="rounded-lg bg-surface-card p-3 outline outline-[0.5px] outline-surface-border"
            key={step.label}
            transition={{
              duration: 1.2,
              repeat:
                step.active && !reducedMotion ? Number.POSITIVE_INFINITY : 0,
            }}
          >
            <div className="flex items-center justify-between gap-3">
              <p className="text-sm font-semibold text-ink-muted">
                {index + 1}. {step.label}
              </p>
              {step.active ? <LiveDots /> : null}
            </div>
            <Badge
              className="mt-3"
              variant={
                step.variant as "success" | "warning" | "error" | "outline"
              }
            >
              {statusLabel(step.value)}
            </Badge>
          </motion.div>
        ))}
      </div>
    </div>
  );
}

function LiveDots() {
  const reducedMotion = useReducedMotion();

  return (
    <span className="flex items-center gap-1" aria-hidden="true">
      {[0, 1, 2].map((item) => (
        <motion.span
          animate={
            reducedMotion
              ? { opacity: 1 }
              : {
                  opacity: [0.35, 1, 0.35],
                  y: [0, -2, 0],
                }
          }
          className="size-1.5 rounded-full bg-ink-muted"
          key={item}
          transition={{
            delay: item * 0.15,
            duration: 0.9,
            repeat: reducedMotion ? 0 : Number.POSITIVE_INFINITY,
          }}
        />
      ))}
    </span>
  );
}

function SettingsView({
  connection,
  persistSettings,
  settings,
  syncing,
  testConnection,
}: {
  connection: ConnectionState;
  persistSettings(settings: BotoApiSettings): void;
  settings: BotoApiSettings;
  syncing: boolean;
  testConnection(): void;
}) {
  const [draft, setDraft] = useState(settings);

  useEffect(() => {
    setDraft(settings);
  }, [settings]);

  return (
    <div className="grid gap-6 xl:grid-cols-[0.9fr_1.1fr]">
      <Card className="p-6">
        <div className="flex items-center justify-between gap-4">
          <div>
            <h2 className="text-xl font-semibold tracking-tight">
              API settings
            </h2>
            <p className="mt-2 text-base leading-relaxed text-ink-secondary">
              Requests are proxied through `/api/backend/...` with tenant and
              actor headers.
            </p>
          </div>
          <Badge
            variant={
              connection === "connected"
                ? "success"
                : connection === "error"
                  ? "error"
                  : "warning"
            }
          >
            {connection}
          </Badge>
        </div>
        <form
          className="mt-6 grid gap-4"
          onSubmit={(event) => {
            event.preventDefault();
            persistSettings(draft);
          }}
        >
          <Field label="Backend URL">
            <input
              className={inputClassName()}
              onChange={(event) =>
                setDraft({ ...draft, backendUrl: event.target.value })
              }
              value={draft.backendUrl}
            />
          </Field>
          <div className="grid gap-4 sm:grid-cols-2">
            <Field label="Tenant ID">
              <input
                className={inputClassName()}
                onChange={(event) =>
                  setDraft({ ...draft, tenantId: event.target.value })
                }
                value={draft.tenantId}
              />
            </Field>
            <Field label="Actor ID">
              <input
                className={inputClassName()}
                onChange={(event) =>
                  setDraft({ ...draft, actorId: event.target.value })
                }
                value={draft.actorId}
              />
            </Field>
          </div>
          <div className="flex flex-wrap gap-3">
            <Button type="submit">
              <ShieldCheck aria-hidden="true" />
              Save and sync
            </Button>
            <Button
              disabled={syncing}
              onClick={testConnection}
              type="button"
              variant="secondary"
            >
              <RefreshCw
                aria-hidden="true"
                className={cn(syncing && "animate-spin")}
              />
              Test connection
            </Button>
          </div>
        </form>
      </Card>
    </div>
  );
}
