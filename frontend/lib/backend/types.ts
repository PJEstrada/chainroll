export type LifecycleMeta = {
  status: "Active" | "Inactive" | "active" | "inactive";
  created: string;
  updated: string;
};

export type Employee = {
  id: string;
  metadata: LifecycleMeta;
  identifier: string;
  first_name: string;
  last_name: string;
  divisions: string[];
  culture: string | null;
  wallet_address: string | null;
  attributes: Record<string, unknown> | null;
};

export type CreateEmployeeInput = {
  identifier: string;
  first_name: string;
  last_name: string;
  wallet_address?: string | null;
};

export type EmployeeCountResponse = {
  count: number;
};

export type TreasuryAccount = {
  id: string;
  tenant_id: string;
  metadata: LifecycleMeta;
  name: string;
  chain: "tempo-testnet";
  token_symbol: string;
  token_address: string;
  token_decimals: number;
  sender_address: string;
  custody_provider: "local_key" | "privy" | "external";
  control_mode:
    | "server_controlled"
    | "user_delegated"
    | "user_signature_required"
    | "external_execution";
  provider_wallet_id: string | null;
  provider_owner_id: string | null;
  secret_reference: string | null;
  is_default: boolean;
};

export type TreasuryAccountInput = {
  name: string;
  chain: "tempo-testnet";
  token_symbol: string;
  token_address: string;
  token_decimals: number;
  sender_address: string;
  custody_provider: "local_key" | "privy" | "external";
  control_mode:
    | "server_controlled"
    | "user_delegated"
    | "user_signature_required"
    | "external_execution";
  provider_wallet_id?: string | null;
  provider_owner_id?: string | null;
  secret_reference?: string | null;
  is_default: boolean;
};

export type CompensationProfile = {
  id: string;
  tenant_id: string;
  employee_id: string;
  metadata: LifecycleMeta;
  amount: {
    amount_units: string;
    token_symbol: string;
  };
  cadence: "weekly" | "biweekly" | "monthly" | "custom";
  valid_from: string | null;
  valid_to: string | null;
};

export type CompensationProfileInput = {
  amount_units: string;
  token_symbol: string;
  cadence: "weekly" | "biweekly" | "monthly" | "custom";
  cadence_every?: number | null;
  cadence_unit?: "days" | "weeks" | "months" | null;
  valid_from?: string | null;
  valid_to?: string | null;
};

export type BotoApiSettings = {
  backendUrl: string;
  tenantId: string;
  actorId: string;
};
