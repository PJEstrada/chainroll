# Boto Frontend

Marketing landing page plus a product dashboard for Boto. The dashboard wires
the working Rust employee, compensation profile, and treasury endpoints through
the Next proxy. Payrun preview calls the live Rust `POST /payruns/preview`
endpoint; payrun execution remains mocked behind the replaceable payroll client
adapter until persistence/submission endpoints land.

## Run

```bash
pnpm install
pnpm dev
```

Open `http://localhost:3000`.

The marketing page is at `/`; the product dashboard is at `/app`.

The dashboard proxies working Rust API endpoints through `/api/backend/...`.
Set the backend URL, tenant ID, and actor ID from the dashboard Settings view.
Workers can be created with an optional compensation profile in the same flow.
Payroll preview loads automatically from the backend and mocked execution
auto-polls until settlement attempts expose Tempo explorer transaction links.

If the Rust API is running on its default `http://localhost:3000`, run the
frontend on another port:

```bash
pnpm dev -- -p 8000
```

## Checks

```bash
pnpm lint
pnpm build
rg -n "#[0-9A-Fa-f]{3,8}" app components lib
```

Only `app/globals.css` should contain color hex values.
