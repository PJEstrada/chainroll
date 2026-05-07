"use client";

import {
  AlertCircle,
  ArrowRight,
  Banknote,
  ExternalLink,
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
  PayrollPreviewStatus,
} from "@/lib/payroll";
import { mockPayrollClient } from "@/lib/payroll";
import { cn } from "@/lib/utils";

type AppView = "overview" | "workers" | "treasury" | "payroll" | "settings";
type ConnectionState = "idle" | "checking" | "connected" | "error";
type WorkerStep = "identity" | "wallet" | "compensation" | "review";
type PayrollTab = "preview" | "exceptions" | "execution";

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

const terminalExecutionStatuses: PayrollExecutionStatus[] = [
  "completed",
  "failed",
  "partially_failed",
  "review_required",
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
  return `${profile.amount.amount_units} ${profile.amount.token_symbol}`;
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

function previewVariant(
  status: PayrollPreviewStatus | undefined,
): "success" | "warning" | "error" | "outline" {
  if (!status) {
    return "outline";
  }

  if (status === "ready") {
    return "success";
  }

  return status === "blocked" ? "error" : "warning";
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
  const [employeeWizard, setEmployeeWizard] = useState(initialEmployeeWizard);
  const [workerStep, setWorkerStep] = useState<WorkerStep>("identity");
  const [workerDrawerOpen, setWorkerDrawerOpen] = useState(false);
  const [compensationDrawerOpen, setCompensationDrawerOpen] = useState(false);
  const [treasuryForm, setTreasuryForm] = useState(initialTreasuryForm);
  const [compensationForm, setCompensationForm] = useState(
    initialCompensationForm,
  );
  const [preview, setPreview] = useState<PayrollPreview | null>(null);
  const [execution, setExecution] = useState<PayrollExecution | null>(null);
  const [payrollTab, setPayrollTab] = useState<PayrollTab>("preview");
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
      if (!selectedEmployeeId && nextEmployees[0]) {
        setSelectedEmployeeId(nextEmployees[0].id);
      }
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
    if (view !== "payroll" || preview) {
      return;
    }

    let canceled = false;
    setPayrollBusy(true);
    void botoApi
      .previewPayrun(settings, employees)
      .then((nextPreview) => {
        if (canceled) {
          return;
        }

        setPreview(nextPreview);
        setPayrollTab("preview");
      })
      .catch((error) => {
        if (!canceled) {
          setMessage(
            error instanceof Error
              ? error.message
              : "Unable to preview payroll",
          );
        }
      })
      .finally(() => {
        if (!canceled) {
          setPayrollBusy(false);
        }
      });

    return () => {
      canceled = true;
    };
  }, [employees, preview, settings, view]);

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
      await botoApi.createTreasuryAccount(settings, {
        ...treasuryForm,
        provider_wallet_id: treasuryForm.provider_wallet_id || null,
        provider_owner_id: treasuryForm.provider_owner_id || null,
        secret_reference: treasuryForm.secret_reference || null,
      });
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

  async function loadPayrollPreview() {
    setPayrollBusy(true);
    setMessage("");

    try {
      const nextPreview = await botoApi.previewPayrun(settings, employees);
      setPreview(nextPreview);
      setExecution(null);
      setPayrollTab("preview");
      setMessage("Payroll preview loaded from the Rust API.");
    } catch (error) {
      setMessage(
        error instanceof Error ? error.message : "Unable to preview payroll",
      );
    } finally {
      setPayrollBusy(false);
    }
  }

  async function executeReadyPayroll() {
    if (!preview) {
      return;
    }

    setPayrollBusy(true);
    setMessage("");

    try {
      const nextExecution = await mockPayrollClient.executePayroll({
        previewId: preview.id,
        preview,
        executeBlocked: false,
      });
      setExecution(nextExecution);
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
                createWorkerFromWizard={createWorkerFromWizard}
                deleteEmployee={deleteEmployee}
                employeeWizard={employeeWizard}
                employees={employees}
                selectedEmployee={selectedEmployee}
                selectedEmployeeId={selectedEmployeeId}
                setCompensationDrawerOpen={setCompensationDrawerOpen}
                setCompensationForm={setCompensationForm}
                setEmployeeWizard={setEmployeeWizard}
                setSelectedEmployeeId={setSelectedEmployeeId}
                setWorkerDrawerOpen={setWorkerDrawerOpen}
                setWorkerStep={setWorkerStep}
                submitCompensation={submitCompensation}
                workerDrawerOpen={workerDrawerOpen}
                workerStep={workerStep}
              />
            ) : null}

            {view === "treasury" ? (
              <TreasuryView
                busy={mutating}
                deactivateTreasuryAccount={deactivateTreasuryAccount}
                setTreasuryForm={setTreasuryForm}
                submitTreasury={submitTreasury}
                treasuryAccounts={treasuryAccounts}
                treasuryForm={treasuryForm}
              />
            ) : null}

            {view === "payroll" ? (
              <PayrollWorkspace
                executeReadyPayroll={executeReadyPayroll}
                execution={execution}
                payrollBusy={payrollBusy}
                payrollTab={payrollTab}
                pollingExecution={pollingExecution}
                preview={preview}
                refreshPreview={loadPayrollPreview}
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
  createWorkerFromWizard,
  deleteEmployee,
  employeeWizard,
  employees,
  selectedEmployee,
  selectedEmployeeId,
  setCompensationDrawerOpen,
  setCompensationForm,
  setEmployeeWizard,
  setSelectedEmployeeId,
  setWorkerDrawerOpen,
  setWorkerStep,
  submitCompensation,
  workerDrawerOpen,
  workerStep,
}: {
  busy: boolean;
  compensationDrawerOpen: boolean;
  compensationForm: CompensationForm;
  compensationProfiles: CompensationProfile[];
  createWorkerFromWizard(event: FormEvent<HTMLFormElement>): Promise<void>;
  deleteEmployee(id: string): Promise<void>;
  employeeWizard: EmployeeWizardForm;
  employees: Employee[];
  selectedEmployee?: Employee;
  selectedEmployeeId: string;
  setCompensationDrawerOpen(open: boolean): void;
  setCompensationForm: Dispatch<SetStateAction<CompensationForm>>;
  setEmployeeWizard: Dispatch<SetStateAction<EmployeeWizardForm>>;
  setSelectedEmployeeId(value: string): void;
  setWorkerDrawerOpen(open: boolean): void;
  setWorkerStep(step: WorkerStep): void;
  submitCompensation(event: FormEvent<HTMLFormElement>): Promise<void>;
  workerDrawerOpen: boolean;
  workerStep: WorkerStep;
}) {
  const activeWorkers = employees.filter(
    (employee) => employee.metadata.status.toLowerCase() === "active",
  ).length;
  const walletReady = employees.filter(
    (employee) => employee.wallet_address,
  ).length;

  return (
    <div className="grid gap-6">
      <div className="grid gap-4 md:grid-cols-3">
        {[
          ["Total workers", employees.length],
          ["Active", activeWorkers],
          ["Wallet-ready", `${walletReady}/${employees.length}`],
        ].map(([label, value]) => (
          <Card className="p-5" key={label}>
            <p className="text-sm font-semibold text-ink-muted">{label}</p>
            <p className="mt-4 font-mono text-2xl font-semibold text-ink-primary">
              {value}
            </p>
          </Card>
        ))}
      </div>

      <Card className="overflow-hidden">
        <div className="flex flex-col gap-4 border-b-[0.5px] border-surface-border p-5 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <div className="flex flex-wrap gap-2">
              <Badge variant="success">GET /employees</Badge>
              <Badge variant="success">POST /employees</Badge>
              <Badge variant="success">POST /compensation-profiles</Badge>
            </div>
            <h2 className="mt-4 text-xl font-semibold tracking-tight">
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
              {employees.map((employee) => (
                <tr
                  className={cn(
                    "border-b-[0.5px] border-surface-border last:border-b-0",
                    selectedEmployeeId === employee.id && "bg-surface-page",
                  )}
                  key={employee.id}
                >
                  <td className="px-5 py-4">
                    <button
                      className="text-left font-semibold text-ink-primary hover:text-brand-primary"
                      onClick={() => setSelectedEmployeeId(employee.id)}
                      type="button"
                    >
                      {employee.first_name} {employee.last_name}
                    </button>
                  </td>
                  <td className="px-5 py-4 font-mono text-sm font-semibold text-ink-muted">
                    {employee.identifier}
                  </td>
                  <td className="px-5 py-4">
                    <Badge
                      variant={employee.wallet_address ? "success" : "warning"}
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
                    {selectedEmployeeId === employee.id &&
                    compensationProfiles.length > 0 ? (
                      <Badge variant="success">
                        {compensationProfiles.length} profile
                      </Badge>
                    ) : (
                      <Badge variant="outline">Select to view</Badge>
                    )}
                  </td>
                  <td className="px-5 py-4">
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
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
          {employees.length === 0 ? (
            <p className="p-5 text-base leading-relaxed text-ink-secondary">
              No workers returned from the API.
            </p>
          ) : null}
        </div>
      </Card>

      <div className="grid gap-6 xl:grid-cols-[0.95fr_1.05fr]">
        <Card className="p-5">
          <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
            <div>
              <h2 className="text-xl font-semibold tracking-tight">
                Worker detail
              </h2>
              <p className="mt-2 text-base leading-relaxed text-ink-secondary">
                {selectedEmployee
                  ? `${selectedEmployee.first_name} ${selectedEmployee.last_name}`
                  : "Select a worker from the roster"}
              </p>
            </div>
            <Badge
              variant={selectedEmployee?.wallet_address ? "success" : "warning"}
            >
              {selectedEmployee?.wallet_address ? "Wallet ready" : "No wallet"}
            </Badge>
          </div>
          {selectedEmployee ? (
            <div className="mt-5 grid gap-3">
              {[
                ["Worker ID", selectedEmployee.id],
                ["Identifier", selectedEmployee.identifier],
                ["Wallet", truncateAddress(selectedEmployee.wallet_address)],
                ["Lifecycle", selectedEmployee.metadata.status],
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
        </Card>

        <Card className="p-5">
          <div className="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
            <div>
              <div className="flex flex-wrap gap-2">
                <Badge variant="success">GET /compensation-profiles</Badge>
                <Badge variant="success">POST /compensation-profiles</Badge>
              </div>
              <h2 className="mt-4 text-xl font-semibold tracking-tight">
                Compensation profiles
              </h2>
            </div>
            <Button
              disabled={!selectedEmployee}
              onClick={() => setCompensationDrawerOpen(true)}
              type="button"
              variant="secondary"
            >
              <Plus aria-hidden="true" />
              Add profile
            </Button>
          </div>
          <div className="mt-5 grid gap-3">
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
            {selectedEmployee && compensationProfiles.length === 0 ? (
              <p className="text-base leading-relaxed text-ink-secondary">
                No compensation profiles returned for this worker.
              </p>
            ) : null}
          </div>
        </Card>
      </div>

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
    </div>
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
  setTreasuryForm,
  submitTreasury,
  treasuryAccounts,
  treasuryForm,
}: {
  busy: boolean;
  deactivateTreasuryAccount(id: string): Promise<void>;
  setTreasuryForm: Dispatch<SetStateAction<TreasuryForm>>;
  submitTreasury(event: FormEvent<HTMLFormElement>): Promise<void>;
  treasuryAccounts: TreasuryAccount[];
  treasuryForm: TreasuryForm;
}) {
  return (
    <div className="grid gap-6 xl:grid-cols-[1.1fr_0.9fr]">
      <Card className="overflow-hidden">
        <div className="border-b-[0.5px] border-surface-border p-5">
          <div className="flex flex-wrap gap-2">
            <Badge variant="success">GET /treasury-accounts</Badge>
            <Badge variant="success">POST /treasury-accounts</Badge>
            <Badge variant="success">DELETE /treasury-accounts/:id</Badge>
          </div>
          <h2 className="mt-4 text-xl font-semibold tracking-tight">
            Treasury accounts
          </h2>
        </div>
        <div className="grid gap-3 p-5">
          {treasuryAccounts.map((account) => (
            <div
              className="rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border"
              key={account.id}
            >
              <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <div>
                  <p className="font-semibold text-ink-primary">
                    {account.name}
                  </p>
                  <p className="mt-1 font-mono text-sm font-semibold text-ink-muted">
                    {account.chain} · {account.token_symbol}
                  </p>
                </div>
                <div className="flex flex-wrap gap-2">
                  <Badge variant={statusVariant(account.metadata.status)}>
                    {account.metadata.status}
                  </Badge>
                  {account.is_default ? (
                    <Badge variant="success">Default</Badge>
                  ) : null}
                  <Badge variant="outline">{account.control_mode}</Badge>
                </div>
              </div>
              <div className="mt-4 grid gap-3 md:grid-cols-2">
                <div className="rounded-lg bg-surface-card p-3 outline outline-[0.5px] outline-surface-border">
                  <p className="text-sm font-semibold text-ink-muted">Sender</p>
                  <p className="mt-2 font-mono text-sm font-semibold text-ink-primary">
                    {truncateAddress(account.sender_address)}
                  </p>
                </div>
                <div className="rounded-lg bg-surface-card p-3 outline outline-[0.5px] outline-surface-border">
                  <p className="text-sm font-semibold text-ink-muted">
                    Custody
                  </p>
                  <p className="mt-2 font-mono text-sm font-semibold text-ink-primary">
                    {account.custody_provider}
                  </p>
                </div>
              </div>
              <Button
                className="mt-4"
                disabled={busy}
                onClick={() => void deactivateTreasuryAccount(account.id)}
                size="sm"
                type="button"
                variant="outline"
              >
                Deactivate
              </Button>
            </div>
          ))}
          {treasuryAccounts.length === 0 ? (
            <p className="text-base leading-relaxed text-ink-secondary">
              No treasury accounts returned from the API.
            </p>
          ) : null}
        </div>
      </Card>

      <Card className="p-5">
        <h2 className="text-xl font-semibold tracking-tight">
          Create treasury
        </h2>
        <form className="mt-5 grid gap-4" onSubmit={submitTreasury}>
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
      </Card>
    </div>
  );
}

function PayrollWorkspace({
  executeReadyPayroll,
  execution,
  payrollBusy,
  payrollTab,
  pollingExecution,
  preview,
  refreshPreview,
  setPayrollTab,
}: {
  executeReadyPayroll(): Promise<void>;
  execution: PayrollExecution | null;
  payrollBusy: boolean;
  payrollTab: PayrollTab;
  pollingExecution: boolean;
  preview: PayrollPreview | null;
  refreshPreview(): Promise<void>;
  setPayrollTab(tab: PayrollTab): void;
}) {
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

  const action = preview
    ? execution
      ? isExecutionTerminal(execution.status)
        ? "Re-run preview"
        : "Submitting automatically"
      : preview.totals.employeesReady > 0
        ? "Execute ready payments"
        : "Review blockers"
    : "Preview payroll";

  function runPrimaryAction() {
    if (!preview || execution) {
      void refreshPreview();
      return;
    }

    if (preview.totals.employeesReady === 0) {
      setPayrollTab("exceptions");
      return;
    }

    void executeReadyPayroll();
  }

  return (
    <div className="grid gap-6" data-testid="payroll-workspace">
      <Card className="overflow-hidden">
        <div className="grid gap-6 border-b-[0.5px] border-surface-border p-5 xl:grid-cols-[1fr_auto] xl:items-center">
          <div>
            <div className="flex flex-wrap gap-2">
              <Badge
                variant={preview?.source === "api" ? "success" : "warning"}
              >
                {preview?.source === "api"
                  ? "Real preview API"
                  : "Loading preview"}
              </Badge>
              <Badge variant="warning">Mocked execution</Badge>
              <Badge variant={previewVariant(preview?.status)}>
                {preview ? statusLabel(preview.status) : "loading preview"}
              </Badge>
              {execution ? (
                <Badge variant={executionVariant(execution.status)}>
                  {statusLabel(execution.status)}
                </Badge>
              ) : null}
              {pollingExecution ? (
                <Badge variant="warning">Auto-refreshing</Badge>
              ) : null}
            </div>
            <h2 className="mt-4 text-2xl font-semibold tracking-tight">
              May payroll run
            </h2>
            <p className="mt-2 text-base leading-relaxed text-ink-secondary">
              Preview workers, clear exceptions, execute ready payments, and
              track settlement attempts.
            </p>
          </div>
          <Button
            disabled={
              payrollBusy ||
              (execution ? !isExecutionTerminal(execution.status) : false)
            }
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

        <div className="grid gap-4 p-5 md:grid-cols-2 xl:grid-cols-5">
          {[
            [
              "Payable",
              preview?.totals.totalAmounts
                .map((amount) => amount.display)
                .join(" + ") ?? "Loading",
            ],
            ["Employees", preview?.totals.totalEmployees ?? "-"],
            ["Ready", preview?.totals.employeesReady ?? "-"],
            ["Blocked", preview?.totals.employeesBlocked ?? "-"],
            [
              "Submitted",
              execution
                ? `${executionCounts.completed} done · ${executionCounts.inTransit} moving`
                : "Not started",
            ],
          ].map(([label, value]) => (
            <div
              className="rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border"
              key={label}
            >
              <p className="text-sm font-semibold text-ink-muted">{label}</p>
              <p className="mt-2 font-mono text-lg font-semibold text-ink-primary">
                {value}
              </p>
            </div>
          ))}
        </div>
      </Card>

      <Card className="overflow-hidden">
        <div className="flex flex-col gap-4 border-b-[0.5px] border-surface-border p-5 lg:flex-row lg:items-center lg:justify-between">
          <div className="flex flex-wrap gap-2">
            {[
              ["preview", "Preview"],
              ["exceptions", "Exceptions"],
              ["execution", "Execution"],
            ].map(([id, label]) => (
              <button
                className={cn(
                  "h-10 rounded-full px-4 text-sm font-semibold outline outline-[0.5px] outline-surface-border transition-colors",
                  payrollTab === id
                    ? "bg-surface-page text-ink-primary"
                    : "text-ink-secondary hover:bg-surface-page hover:text-ink-primary",
                )}
                key={id}
                onClick={() => setPayrollTab(id as PayrollTab)}
                type="button"
              >
                {label}
              </button>
            ))}
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="outline">Tempo testnet</Badge>
            <Badge variant="outline">Run date 2026-05-15</Badge>
          </div>
        </div>

        {payrollTab === "preview" ? (
          <PayrollPreviewTable
            attemptsByEmployee={attemptsByEmployee}
            preview={preview}
          />
        ) : null}

        {payrollTab === "exceptions" ? (
          <PayrollExceptions preview={preview} />
        ) : null}

        {payrollTab === "execution" ? (
          <PayrollExecutionPanel
            execution={execution}
            executionCounts={executionCounts}
            pollingExecution={pollingExecution}
          />
        ) : null}
      </Card>
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

function PayrollExceptions({ preview }: { preview: PayrollPreview | null }) {
  const blocked =
    preview?.items.filter((item) => item.blockers.length > 0) ?? [];

  return (
    <div className="grid gap-4 p-5">
      {blocked.map((item) => (
        <div
          className="rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border"
          key={item.employeeId}
        >
          <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
            <div>
              <p className="font-semibold text-ink-primary">
                {item.workerName}
              </p>
              <p className="mt-1 font-mono text-sm font-semibold text-ink-muted">
                {item.amount?.display ?? "No active compensation"}
              </p>
            </div>
            <Badge variant={itemVariant(item)}>
              {item.blockers.map(formatBlocker).join(", ")}
            </Badge>
          </div>
          <div className="mt-4 grid gap-3 md:grid-cols-2">
            {item.blockers.map((blocker) => (
              <div
                className="rounded-lg bg-surface-card p-3 outline outline-[0.5px] outline-surface-border"
                key={blocker}
              >
                <p className="text-sm font-semibold text-ink-muted">
                  {formatBlocker(blocker)}
                </p>
                <p className="mt-2 text-base leading-relaxed text-ink-secondary">
                  {blockerResolution(blocker)}
                </p>
              </div>
            ))}
          </div>
        </div>
      ))}
      {blocked.length === 0 ? (
        <div className="rounded-lg bg-surface-page p-5 outline outline-[0.5px] outline-surface-border">
          <Badge variant="success">No blockers</Badge>
        </div>
      ) : null}
    </div>
  );
}

function PayrollExecutionPanel({
  execution,
  executionCounts,
  pollingExecution,
}: {
  execution: PayrollExecution | null;
  executionCounts: { completed: number; failed: number; inTransit: number };
  pollingExecution: boolean;
}) {
  if (!execution) {
    return (
      <div className="p-5">
        <Badge variant="outline">Execution not started</Badge>
      </div>
    );
  }

  return (
    <div className="grid gap-5 p-5">
      <div className="grid gap-4 md:grid-cols-3">
        {[
          ["Completed", executionCounts.completed, "success"],
          ["In transit", executionCounts.inTransit, "warning"],
          ["Failed", executionCounts.failed, "error"],
        ].map(([label, value, variant]) => (
          <div
            className="rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border"
            key={label}
          >
            <div className="flex items-center justify-between">
              <p className="text-sm font-semibold text-ink-muted">{label}</p>
              <Badge variant={variant as "success" | "warning" | "error"}>
                {value}
              </Badge>
            </div>
          </div>
        ))}
      </div>

      <div className="rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border">
        <div className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
          <div>
            <p className="font-semibold text-ink-primary">Execution rail</p>
            <p className="mt-1 font-mono text-sm font-semibold text-ink-muted">
              {execution.id}
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Badge variant={executionVariant(execution.status)}>
              {statusLabel(execution.status)}
            </Badge>
            {pollingExecution ? <Badge variant="warning">Polling</Badge> : null}
          </div>
        </div>
        <div className="mt-5 grid gap-3 lg:grid-cols-5">
          {[
            ["Preview", "completed"],
            ["Review blockers", "completed"],
            ["Create payrun", "completed"],
            ["Submit payouts", execution.status],
            ["Settlement", execution.status],
          ].map(([label, value], index) => (
            <div
              className="rounded-lg bg-surface-card p-3 outline outline-[0.5px] outline-surface-border"
              key={label}
            >
              <p className="text-sm font-semibold text-ink-muted">
                {index + 1}. {label}
              </p>
              <p className="mt-2 font-mono text-sm font-semibold text-ink-primary">
                {statusLabel(value)}
              </p>
            </div>
          ))}
        </div>
      </div>

      <div className="overflow-x-auto rounded-lg outline outline-[0.5px] outline-surface-border">
        <table className="w-full min-w-[720px] text-left">
          <thead>
            <tr className="border-b-[0.5px] border-surface-border text-sm font-semibold text-ink-secondary">
              <th className="px-4 py-3">Worker</th>
              <th className="px-4 py-3">Amount</th>
              <th className="px-4 py-3">Status</th>
              <th className="px-4 py-3">Transaction</th>
            </tr>
          </thead>
          <tbody>
            {execution.attempts.map((attempt) => (
              <tr
                className="border-b-[0.5px] border-surface-border last:border-b-0"
                key={attempt.id}
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
                  {attempt.transactionHash ? (
                    <a
                      className="inline-flex items-center gap-2 rounded-full px-3 py-2 text-sm font-semibold text-ink-primary outline outline-[0.5px] outline-surface-border hover:bg-surface-card"
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
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
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

      <Card className="p-6">
        <h2 className="text-xl font-semibold tracking-tight">Endpoint map</h2>
        <div className="mt-5 grid gap-3">
          {[
            [
              "Workers",
              "GET /employees · POST /employees · DELETE /employees/:id",
            ],
            [
              "Compensation",
              "GET /employees/:id/compensation-profiles · POST /employees/:id/compensation-profiles",
            ],
            [
              "Treasury",
              "GET /treasury-accounts · POST /treasury-accounts · DELETE /treasury-accounts/:id",
            ],
            [
              "Payroll",
              "POST /payruns/preview is live · execution remains mocked",
            ],
          ].map(([label, value]) => (
            <div
              className="rounded-lg bg-surface-page p-4 outline outline-[0.5px] outline-surface-border"
              key={label}
            >
              <p className="text-sm font-semibold text-ink-muted">{label}</p>
              <p className="mt-2 font-mono text-sm font-semibold text-ink-primary">
                {value}
              </p>
            </div>
          ))}
        </div>
      </Card>
    </div>
  );
}
