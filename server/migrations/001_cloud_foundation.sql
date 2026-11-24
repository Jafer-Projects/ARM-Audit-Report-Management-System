CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS schema_migrations (
  version text PRIMARY KEY,
  applied_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE users (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  email text NOT NULL,
  email_normalized text GENERATED ALWAYS AS (lower(btrim(email))) STORED,
  display_name text NOT NULL CHECK (length(btrim(display_name)) BETWEEN 1 AND 160),
  password_hash text NOT NULL,
  status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled', 'pending_verification')),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (email_normalized)
);

CREATE TABLE tenants (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  name text NOT NULL CHECK (length(btrim(name)) BETWEEN 1 AND 200),
  slug text NOT NULL UNIQUE,
  created_by uuid NOT NULL REFERENCES users(id),
  status text NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'closed')),
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE tenant_memberships (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL REFERENCES tenants(id) ON DELETE RESTRICT,
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
  role text NOT NULL CHECK (role IN ('owner', 'firm_admin', 'manager', 'professional', 'client', 'platform_support')),
  status text NOT NULL DEFAULT 'active' CHECK (status IN ('invited', 'active', 'suspended', 'revoked')),
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, user_id)
);
CREATE INDEX tenant_memberships_user_idx ON tenant_memberships(user_id, status);

CREATE TABLE auth_sessions (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  refresh_token_hash text NOT NULL,
  expires_at timestamptz NOT NULL,
  revoked_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  rotated_at timestamptz
);
CREATE INDEX auth_sessions_user_idx ON auth_sessions(user_id, expires_at);

ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE users FORCE ROW LEVEL SECURITY;
CREATE POLICY users_self_scope ON users
  USING (id::text = current_setting('app.user_id', true))
  WITH CHECK (id::text = current_setting('app.user_id', true));

ALTER TABLE auth_sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE auth_sessions FORCE ROW LEVEL SECURITY;
CREATE POLICY sessions_self_scope ON auth_sessions
  USING (user_id::text = current_setting('app.user_id', true))
  WITH CHECK (user_id::text = current_setting('app.user_id', true));

CREATE OR REPLACE FUNCTION public.arm_lookup_user_for_login(p_email text)
RETURNS TABLE(user_id uuid, email text, display_name text, password_hash text, status text)
LANGUAGE sql SECURITY DEFINER SET search_path = pg_catalog, public AS $fn$
  SELECT u.id, u.email, u.display_name, u.password_hash, u.status
  FROM public.users u
  WHERE u.email_normalized = lower(btrim(p_email))
  LIMIT 1
$fn$;

CREATE OR REPLACE FUNCTION public.arm_lookup_session_for_refresh(p_token_hash text)
RETURNS TABLE(session_id uuid, user_id uuid)
LANGUAGE sql SECURITY DEFINER SET search_path = pg_catalog, public AS $fn$
  SELECT s.id, s.user_id
  FROM public.auth_sessions s
  JOIN public.users u ON u.id = s.user_id
  WHERE s.refresh_token_hash = p_token_hash
    AND s.revoked_at IS NULL AND s.expires_at > now() AND u.status = 'active'
  LIMIT 1
$fn$;

REVOKE ALL ON FUNCTION public.arm_lookup_user_for_login(text) FROM PUBLIC;
REVOKE ALL ON FUNCTION public.arm_lookup_session_for_refresh(text) FROM PUBLIC;


CREATE TABLE sync_aggregates (
  tenant_id uuid NOT NULL REFERENCES tenants(id) ON DELETE RESTRICT,
  aggregate_type text NOT NULL CHECK (length(aggregate_type) BETWEEN 1 AND 80),
  aggregate_id uuid NOT NULL,
  version bigint NOT NULL CHECK (version > 0),
  payload jsonb NOT NULL,
  updated_by uuid NOT NULL REFERENCES users(id),
  updated_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, aggregate_type, aggregate_id)
);

CREATE TABLE sync_inbox (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL REFERENCES tenants(id) ON DELETE RESTRICT,
  actor_id uuid NOT NULL REFERENCES users(id),
  event_id uuid NOT NULL,
  idempotency_key text NOT NULL,
  aggregate_type text NOT NULL,
  aggregate_id uuid NOT NULL,
  base_version bigint NOT NULL CHECK (base_version >= 0),
  resulting_version bigint,
  event_type text NOT NULL,
  schema_version integer NOT NULL CHECK (schema_version > 0),
  payload jsonb NOT NULL,
  outcome text NOT NULL CHECK (outcome IN ('accepted', 'conflict', 'rejected')),
  outcome_detail jsonb NOT NULL DEFAULT '{}'::jsonb,
  received_at timestamptz NOT NULL DEFAULT now(),
  received_seq bigserial NOT NULL UNIQUE,
  UNIQUE (tenant_id, idempotency_key),
  UNIQUE (tenant_id, event_id)
);
CREATE INDEX sync_inbox_received_idx ON sync_inbox(tenant_id, received_seq);

CREATE TABLE audit_events (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL REFERENCES tenants(id) ON DELETE RESTRICT,
  actor_id uuid NOT NULL REFERENCES users(id),
  action text NOT NULL,
  target_type text NOT NULL,
  target_id text NOT NULL,
  correlation_id text,
  metadata jsonb NOT NULL DEFAULT '{}'::jsonb,
  occurred_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX audit_events_tenant_time_idx ON audit_events(tenant_id, occurred_at DESC);

CREATE TABLE jobs (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid REFERENCES tenants(id) ON DELETE RESTRICT,
  job_type text NOT NULL,
  payload jsonb NOT NULL,
  status text NOT NULL DEFAULT 'queued' CHECK (status IN ('queued', 'running', 'completed', 'failed')),
  attempts integer NOT NULL DEFAULT 0 CHECK (attempts >= 0),
  available_at timestamptz NOT NULL DEFAULT now(),
  locked_at timestamptz,
  locked_by text,
  last_error text,
  created_at timestamptz NOT NULL DEFAULT now(),
  completed_at timestamptz
);
CREATE INDEX jobs_claim_idx ON jobs(status, available_at, created_at);

ALTER TABLE tenants ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenants FORCE ROW LEVEL SECURITY;
CREATE POLICY tenants_select ON tenants FOR SELECT
  USING (id::text = current_setting('app.tenant_id', true) OR created_by::text = current_setting('app.user_id', true));
CREATE POLICY tenants_insert ON tenants FOR INSERT
  WITH CHECK (created_by::text = current_setting('app.user_id', true));

ALTER TABLE tenant_memberships ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_memberships FORCE ROW LEVEL SECURITY;
CREATE POLICY memberships_tenant_scope ON tenant_memberships
  USING (user_id::text = current_setting('app.user_id', true))
  WITH CHECK (
    tenant_id::text = current_setting('app.tenant_id', true)
    AND user_id::text = current_setting('app.user_id', true)
  );

DO $arm$
DECLARE table_name text;
BEGIN
  FOREACH table_name IN ARRAY ARRAY['sync_aggregates','sync_inbox','audit_events','jobs'] LOOP
    EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY', table_name);
    EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY', table_name);
    EXECUTE format(
      'CREATE POLICY %I ON %I USING (tenant_id::text = current_setting(''app.tenant_id'', true) AND EXISTS (SELECT 1 FROM tenant_memberships m WHERE m.tenant_id = %I.tenant_id AND m.user_id::text = current_setting(''app.user_id'', true) AND m.status = ''active'')) WITH CHECK (tenant_id::text = current_setting(''app.tenant_id'', true) AND EXISTS (SELECT 1 FROM tenant_memberships m WHERE m.tenant_id = %I.tenant_id AND m.user_id::text = current_setting(''app.user_id'', true) AND m.status = ''active''))',
      table_name || '_tenant_scope', table_name, table_name, table_name
    );
  END LOOP;
END $arm$;

CREATE POLICY jobs_worker_access ON jobs
  USING (current_setting('app.worker', true) = 'true')
  WITH CHECK (current_setting('app.worker', true) = 'true');

CREATE OR REPLACE FUNCTION reject_immutable_row_mutation() RETURNS trigger
LANGUAGE plpgsql AS $fn$
BEGIN
  RAISE EXCEPTION 'table % is append-only', TG_TABLE_NAME USING ERRCODE = '55000';
END;
$fn$;
CREATE TRIGGER audit_events_append_only
  BEFORE UPDATE OR DELETE ON audit_events
  FOR EACH ROW EXECUTE FUNCTION reject_immutable_row_mutation();
CREATE TRIGGER sync_inbox_append_only
  BEFORE UPDATE OR DELETE ON sync_inbox
  FOR EACH ROW EXECUTE FUNCTION reject_immutable_row_mutation();

-- Runtime writes to the audit log are append-only by application contract.
-- Production database roles must not be granted UPDATE/DELETE on audit_events.

DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'arm_runtime') THEN
    REVOKE ALL ON jobs FROM arm_runtime;
    REVOKE ALL ON users, auth_sessions FROM arm_runtime;
    GRANT SELECT, INSERT ON users TO arm_runtime;
    GRANT SELECT, INSERT, UPDATE ON auth_sessions TO arm_runtime;
    GRANT EXECUTE ON FUNCTION public.arm_lookup_user_for_login(text) TO arm_runtime;
    GRANT EXECUTE ON FUNCTION public.arm_lookup_session_for_refresh(text) TO arm_runtime;
    REVOKE UPDATE, DELETE ON audit_events, sync_inbox FROM arm_runtime;
  END IF;
  IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'arm_worker') THEN
    GRANT SELECT, UPDATE ON jobs TO arm_worker;
  END IF;
END $$;
