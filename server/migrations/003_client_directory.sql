-- Tenant-owned client directory and immutable client history.
-- Existing SQLite demo data is intentionally not imported by this migration.
CREATE TABLE clients (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL REFERENCES tenants(id) ON DELETE RESTRICT,
  record_kind text NOT NULL CHECK (record_kind IN ('business','individual','other')),
  legal_name text NOT NULL CHECK (length(btrim(legal_name)) BETWEEN 2 AND 240),
  display_name text NOT NULL CHECK (length(btrim(display_name)) BETWEEN 1 AND 240),
  tin_raw text NOT NULL DEFAULT '',
  sector text NOT NULL DEFAULT '',
  contact_name text NOT NULL DEFAULT '',
  contact_email text NOT NULL DEFAULT '',
  contact_phone text NOT NULL DEFAULT '',
  address_text text NOT NULL DEFAULT '',
  internal_reference text NOT NULL DEFAULT '',
  notes text NOT NULL DEFAULT '',
  status text NOT NULL DEFAULT 'active' CHECK (status IN ('active','inactive','archived')),
  row_version bigint NOT NULL DEFAULT 1 CHECK (row_version > 0),
  created_by uuid NOT NULL REFERENCES users(id),
  updated_by uuid NOT NULL REFERENCES users(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (tenant_id,id)
);
CREATE INDEX clients_tenant_status_name_idx ON clients(tenant_id,status,lower(legal_name));
CREATE INDEX clients_tenant_tin_idx ON clients(tenant_id,lower(btrim(tin_raw))) WHERE btrim(tin_raw) <> '';
CREATE UNIQUE INDEX clients_tenant_internal_reference_unique ON clients(tenant_id,lower(btrim(internal_reference))) WHERE btrim(internal_reference) <> '';

CREATE TABLE client_history (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  client_id uuid NOT NULL,
  actor_id uuid NOT NULL REFERENCES users(id),
  action text NOT NULL CHECK(action IN ('created','updated','archived','activated','restored')),
  before_data jsonb,
  after_data jsonb NOT NULL,
  client_version bigint NOT NULL CHECK(client_version > 0),
  occurred_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY(tenant_id,client_id) REFERENCES clients(tenant_id,id) ON DELETE RESTRICT
);
CREATE INDEX client_history_tenant_client_time_idx ON client_history(tenant_id,client_id,occurred_at DESC);

ALTER TABLE clients ENABLE ROW LEVEL SECURITY;
ALTER TABLE client_history ENABLE ROW LEVEL SECURITY;
CREATE POLICY clients_tenant_scope ON clients
 USING (tenant_id::text = current_setting('app.tenant_id',true))
 WITH CHECK (tenant_id::text = current_setting('app.tenant_id',true));
CREATE POLICY client_history_tenant_scope ON client_history
 USING (tenant_id::text = current_setting('app.tenant_id',true))
 WITH CHECK (tenant_id::text = current_setting('app.tenant_id',true));

INSERT INTO permissions(permission_key,description) VALUES
 ('clients.archive','Archive or restore client records')
ON CONFLICT DO NOTHING;
