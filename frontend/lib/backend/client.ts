import type {
  BotoApiSettings,
  CompensationProfile,
  CompensationProfileInput,
  CreateEmployeeInput,
  Employee,
  EmployeeCountResponse,
  TreasuryAccount,
  TreasuryAccountInput,
} from "@/lib/backend/types";
import type {
  PayrollPreview,
  PayrollPreviewBlocker,
  PayrollPreviewStatus,
  TokenAmount,
} from "@/lib/payroll/types";

const DEFAULT_SETTINGS: BotoApiSettings = {
  backendUrl: "http://localhost:3001",
  tenantId: "000000000003V",
  actorId: "000000000003V",
};

export const botoSettingsStorageKey = "boto-api-settings";

export function defaultBotoApiSettings(): BotoApiSettings {
  return DEFAULT_SETTINGS;
}

type RequestOptions = {
  method?: "GET" | "POST" | "PUT" | "DELETE";
  body?: unknown;
  actorRequired?: boolean;
};

type BackendPayrunPreview = {
  tenant_id: string;
  status: PayrollPreviewStatus;
  items: BackendPayrunPreviewItem[];
  totals: {
    total_amounts: BackendTokenAmount[];
    total_blockers: number;
    total_employees: number;
    total_employees_with_blockers: number;
    total_employees_without_blockers: number;
  };
};

type BackendPayrunPreviewItem = {
  employee_id: string;
  amount: BackendTokenAmount | null;
  blockers: PayrollPreviewBlocker[];
};

type BackendTokenAmount = {
  amount_units: string;
  token_symbol: string;
};

export class BotoApiError extends Error {
  constructor(
    message: string,
    public readonly status: number,
  ) {
    super(message);
    this.name = "BotoApiError";
  }
}

async function request<T>(
  settings: BotoApiSettings,
  path: string,
  options: RequestOptions = {},
): Promise<T> {
  const headers = new Headers({
    "content-type": "application/json",
    "x-boto-backend-url": settings.backendUrl,
    "x-tenant-id": settings.tenantId,
  });

  if (options.actorRequired) {
    headers.set("x-actor-id", settings.actorId);
  }

  const response = await fetch(`/api/backend/${path.replace(/^\//, "")}`, {
    method: options.method ?? "GET",
    headers,
    body: options.body ? JSON.stringify(options.body) : undefined,
  });

  if (response.status === 204) {
    return undefined as T;
  }

  const contentType = response.headers.get("content-type") ?? "";
  const payload = contentType.includes("application/json")
    ? await response.json()
    : await response.text();

  if (!response.ok) {
    const message =
      typeof payload === "object" && payload && "error" in payload
        ? String(payload.error)
        : `Request failed with status ${response.status}`;
    throw new BotoApiError(message, response.status);
  }

  return payload as T;
}

function amountDisplay(amount: BackendTokenAmount): string {
  const normalized = amount.amount_units.replace(/^0+/, "") || "0";
  const grouped = normalized.replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  return `${grouped} ${amount.token_symbol}`;
}

function normalizeAmount(amount: BackendTokenAmount): TokenAmount {
  return {
    amountUnits: amount.amount_units,
    tokenSymbol: amount.token_symbol,
    display: amountDisplay(amount),
  };
}

function normalizePayrunPreview(
  preview: BackendPayrunPreview,
  employees: Employee[],
): PayrollPreview {
  const employeesById = new Map(
    employees.map((employee) => [employee.id, employee]),
  );

  return {
    id: `preview_${preview.tenant_id}`,
    tenantId: preview.tenant_id,
    source: "api",
    status: preview.status,
    createdAt: new Date().toISOString(),
    totals: {
      totalAmounts: preview.totals.total_amounts.map(normalizeAmount),
      totalBlockers: preview.totals.total_blockers,
      totalEmployees: preview.totals.total_employees,
      employeesReady: preview.totals.total_employees_without_blockers,
      employeesBlocked: preview.totals.total_employees_with_blockers,
    },
    items: preview.items.map((item) => {
      const employee = employeesById.get(item.employee_id);
      const amount = item.amount ? normalizeAmount(item.amount) : null;

      return {
        employeeId: item.employee_id,
        identifier: employee?.identifier ?? item.employee_id,
        workerName: employee
          ? `${employee.first_name} ${employee.last_name}`
          : `Worker ${item.employee_id}`,
        walletAddress: employee?.wallet_address ?? null,
        amount,
        cadence: "custom",
        cadenceLabel: amount ? "Active profile" : "No active profile",
        treasury: amount
          ? {
              chain: "tempo-testnet",
              tokenSymbol: amount.tokenSymbol,
              controlMode: item.blockers.includes(
                "treasury_requires_user_signature",
              )
                ? "user_signature_required"
                : "server_controlled",
              matchesDefault: !item.blockers.includes(
                "missing_treasury_account",
              ),
            }
          : null,
        blockers: item.blockers,
      };
    }),
  };
}

export const botoApi = {
  listEmployees(settings: BotoApiSettings) {
    return request<Employee[]>(settings, "employees?limit=50&offset=0");
  },
  countEmployees(settings: BotoApiSettings) {
    return request<EmployeeCountResponse>(settings, "employees/count").then(
      (response) => response.count,
    );
  },
  async previewPayrun(settings: BotoApiSettings, employees: Employee[] = []) {
    const preview = await request<BackendPayrunPreview>(
      settings,
      "payruns/preview",
      {
        method: "POST",
      },
    );

    return normalizePayrunPreview(preview, employees);
  },
  createEmployee(settings: BotoApiSettings, input: CreateEmployeeInput) {
    return request<Employee>(settings, "employees", {
      method: "POST",
      body: input,
    });
  },
  deleteEmployee(settings: BotoApiSettings, id: string) {
    return request<void>(settings, `employees/${id}`, {
      method: "DELETE",
    });
  },
  listTreasuryAccounts(settings: BotoApiSettings) {
    return request<TreasuryAccount[]>(
      settings,
      "treasury-accounts?limit=50&offset=0",
    );
  },
  createTreasuryAccount(
    settings: BotoApiSettings,
    input: TreasuryAccountInput,
  ) {
    return request<TreasuryAccount>(settings, "treasury-accounts", {
      method: "POST",
      body: input,
      actorRequired: true,
    });
  },
  deactivateTreasuryAccount(settings: BotoApiSettings, id: string) {
    return request<TreasuryAccount>(settings, `treasury-accounts/${id}`, {
      method: "DELETE",
      actorRequired: true,
    });
  },
  listCompensationProfiles(settings: BotoApiSettings, employeeId: string) {
    return request<CompensationProfile[]>(
      settings,
      `employees/${employeeId}/compensation-profiles`,
    );
  },
  createCompensationProfile(
    settings: BotoApiSettings,
    employeeId: string,
    input: CompensationProfileInput,
  ) {
    return request<CompensationProfile>(
      settings,
      `employees/${employeeId}/compensation-profiles`,
      {
        method: "POST",
        body: input,
        actorRequired: true,
      },
    );
  },
};
