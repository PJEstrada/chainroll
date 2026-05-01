-- Add migration script here
ALTER TABLE employees
    ADD COLUMN IF NOT EXISTS wallet_address TEXT;