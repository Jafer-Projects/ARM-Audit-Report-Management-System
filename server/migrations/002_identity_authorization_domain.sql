-- Cloud security and domain foundation v2.
-- Apply only through the dedicated arm_migrator role. Existing records are preserved.

ALTER TABLE users ALTER COLUMN status SET DEFAULT 'pending_verification';
CREATE TABLE email_verification_tokens (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token_hash text NOT NULL UNIQUE,
  expires_at timestamptz NOT NULL,
  consumed_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX email_verification_user_idx ON email_verification_tokens(user_id, expires_at);
CREATE TABLE password_recovery_tokens (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token_hash text NOT NULL UNIQUE,
  expires_at timestamptz NOT NULL,
  consumed_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE mfa_credentials (
  user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  secret_ciphertext text NOT NULL,
  enabled_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE mfa_recovery_codes (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  code_hash text NOT NULL,
  used_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX mfa_recovery_user_idx ON mfa_recovery_codes(user_id) WHERE used_at IS NULL;

CREATE TABLE tenant_roles (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL REFERENCES tenants(id) ON DELETE RESTRICT,
  role_key text NOT NULL CHECK(role_key ~ '^[a-z][a-z0-9_.-]{1,63}$'),
  display_name text NOT NULL CHECK(length(btrim(display_name)) BETWEEN 2 AND 100),
  is_system boolean NOT NULL DEFAULT false,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(tenant_id, role_key),
  UNIQUE(tenant_id, id)
);
CREATE TABLE permissions (
  permission_key text PRIMARY KEY,
  description text NOT NULL
);
CREATE TABLE tenant_role_permissions (
  tenant_id uuid NOT NULL,
  role_id uuid NOT NULL,
  permission_key text NOT NULL REFERENCES permissions(permission_key),
  granted_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY(role_id, permission_key),
  FOREIGN KEY(tenant_id, role_id) REFERENCES tenant_roles(tenant_id,id) ON DELETE CASCADE
);
CREATE TABLE tenant_membership_roles (
  tenant_id uuid NOT NULL,
  membership_id uuid NOT NULL REFERENCES tenant_memberships(id) ON DELETE CASCADE,
  role_id uuid NOT NULL,
  assigned_by uuid NOT NULL REFERENCES users(id),
  assigned_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY(membership_id, role_id),
  FOREIGN KEY(tenant_id, role_id) REFERENCES tenant_roles(tenant_id,id) ON DELETE RESTRICT
);

INSERT INTO permissions(permission_key,description) VALUES
 ('tenant.read','Read tenant workspace'),('tenant.manage','Manage tenant settings and members'),
 ('members.read','Read member directory'),('members.invite','Invite members'),
 ('roles.read','Read role definitions'),('roles.manage','Create roles and assign permissions'),
 ('clients.read','Read client records'),('clients.write','Create and update client records'),
 ('engagements.read','Read audit engagements'),('engagements.manage','Manage audit engagements'),
 ('reports.read','Read audit reports'),('reports.prepare','Prepare audit reports'),
 ('reports.approve','Independently review and approve audit reports'),
 ('workpapers.read','Read audit workpapers'),('workpapers.write','Create and update workpapers'),
 ('accounting.read','Read accounting records'),('accounting.post','Post accounting journals'),
 ('documents.read','Read documents'),('documents.upload','Upload evidence documents'),
 ('documents.release','Release clean documents'),('auditlog.read','Read immutable audit history'),
 ('billing.manage','Manage billing and subscription settings')
ON CONFLICT DO NOTHING;

CREATE TABLE organization_invitations (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL REFERENCES tenants(id) ON DELETE RESTRICT,
  email_normalized text NOT NULL,
  role_id uuid NOT NULL,
  token_hash text NOT NULL UNIQUE,
  invited_by uuid NOT NULL REFERENCES users(id),
  status text NOT NULL DEFAULT 'pending' CHECK(status IN ('pending','accepted','revoked','expired')),
  expires_at timestamptz NOT NULL,
  accepted_by uuid REFERENCES users(id),
  accepted_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY(tenant_id,role_id) REFERENCES tenant_roles(tenant_id,id) ON DELETE RESTRICT
);
CREATE INDEX organization_invitations_tenant_idx ON organization_invitations(tenant_id,email_normalized,status);


CREATE OR REPLACE FUNCTION public.arm_accept_invitation(p_hash text,p_user uuid)
RETURNS TABLE(tenant_id uuid,membership_id uuid)
LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog,public AS $fn$
DECLARE inv public.organization_invitations%ROWTYPE; new_membership uuid;
BEGIN
 IF current_setting('app.user_id',true) IS DISTINCT FROM p_user::text THEN RAISE EXCEPTION 'caller identity mismatch' USING ERRCODE='42501'; END IF;
 SELECT * INTO inv FROM public.organization_invitations
 WHERE token_hash=p_hash AND status='pending' AND expires_at>now() FOR UPDATE;
 IF NOT FOUND THEN RAISE EXCEPTION 'invitation invalid or expired' USING ERRCODE='P0002'; END IF;
 IF NOT EXISTS(SELECT 1 FROM public.users u WHERE u.id=p_user AND u.status='active' AND u.email_normalized=inv.email_normalized) THEN RAISE EXCEPTION 'verified invitee email required' USING ERRCODE='42501'; END IF;
 INSERT INTO public.tenant_memberships(tenant_id,user_id,role,status)
 VALUES(inv.tenant_id,p_user,'professional','active')
 ON CONFLICT(tenant_id,user_id) DO UPDATE SET status='active'
 RETURNING id INTO new_membership;
 INSERT INTO public.tenant_membership_roles(tenant_id,membership_id,role_id,assigned_by)
 VALUES(inv.tenant_id,new_membership,inv.role_id,inv.invited_by)
 ON CONFLICT DO NOTHING;
 UPDATE public.organization_invitations SET status='accepted',accepted_by=p_user,accepted_at=now() WHERE id=inv.id;
 RETURN QUERY SELECT inv.tenant_id,new_membership;
END $fn$;
REVOKE ALL ON FUNCTION public.arm_accept_invitation(text,uuid) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.arm_accept_invitation(text,uuid) TO arm_runtime;

CREATE TABLE audit_engagements (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL REFERENCES tenants(id) ON DELETE RESTRICT,
  client_reference text NOT NULL,
  title text NOT NULL CHECK(length(btrim(title)) BETWEEN 3 AND 200),
  period_start date NOT NULL,
  period_end date NOT NULL CHECK(period_end >= period_start),
  framework text NOT NULL,
  status text NOT NULL DEFAULT 'planning' CHECK(status IN ('planning','fieldwork','review','reporting','completed','archived')),
  version bigint NOT NULL DEFAULT 1 CHECK(version > 0),
  created_by uuid NOT NULL REFERENCES users(id),
  updated_by uuid NOT NULL REFERENCES users(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(tenant_id,id),
  UNIQUE(tenant_id,client_reference)
);
CREATE TABLE audit_workpapers (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  engagement_id uuid NOT NULL,
  reference text NOT NULL,
  title text NOT NULL,
  procedure text NOT NULL,
  conclusion text NOT NULL DEFAULT '',
  status text NOT NULL DEFAULT 'draft' CHECK(status IN ('draft','prepared','reviewed','approved','reopened')),
  version bigint NOT NULL DEFAULT 1 CHECK(version > 0),
  prepared_by uuid NOT NULL REFERENCES users(id),
  reviewed_by uuid REFERENCES users(id),
  approved_by uuid REFERENCES users(id),
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY(tenant_id,engagement_id) REFERENCES audit_engagements(tenant_id,id) ON DELETE RESTRICT,
  UNIQUE(tenant_id,id),
  UNIQUE(tenant_id,engagement_id,reference),
  CHECK(approved_by IS NULL OR (reviewed_by IS NOT NULL AND reviewed_by <> prepared_by)),
  CHECK(reviewed_by IS NULL OR reviewed_by <> prepared_by)
);
CREATE TABLE audit_reports (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  engagement_id uuid NOT NULL,
  report_type text NOT NULL CHECK(report_type IN ('independent_auditor','management_letter','internal_audit','special_purpose')),
  title text NOT NULL,
  body jsonb NOT NULL,
  status text NOT NULL DEFAULT 'draft' CHECK(status IN ('draft','prepared','under_review','approved','issued','withdrawn')),
  version bigint NOT NULL DEFAULT 1 CHECK(version > 0),
  prepared_by uuid NOT NULL REFERENCES users(id),
  reviewed_by uuid REFERENCES users(id),
  approved_by uuid REFERENCES users(id),
  issued_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY(tenant_id,engagement_id) REFERENCES audit_engagements(tenant_id,id) ON DELETE RESTRICT,
  UNIQUE(tenant_id,id),
  CHECK(approved_by IS NULL OR (reviewed_by IS NOT NULL AND reviewed_by <> prepared_by)),
  CHECK(reviewed_by IS NULL OR reviewed_by <> prepared_by)
);
CREATE TABLE accounting_accounts (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL REFERENCES tenants(id) ON DELETE RESTRICT,
  account_code text NOT NULL,
  account_name text NOT NULL,
  account_type text NOT NULL CHECK(account_type IN ('asset','liability','equity','income','expense')),
  currency char(3) NOT NULL,
  active boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(tenant_id,id),
  UNIQUE(tenant_id,account_code)
);
CREATE TABLE accounting_journals (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL REFERENCES tenants(id) ON DELETE RESTRICT,
  journal_number text NOT NULL,
  effective_date date NOT NULL,
  description text NOT NULL,
  status text NOT NULL DEFAULT 'draft' CHECK(status IN ('draft','posted','reversed')),
  currency char(3) NOT NULL,
  total_debit numeric(20,4) NOT NULL DEFAULT 0,
  total_credit numeric(20,4) NOT NULL DEFAULT 0,
  created_by uuid NOT NULL REFERENCES users(id),
  posted_by uuid REFERENCES users(id),
  posted_at timestamptz,
  reversal_of uuid,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(tenant_id,id),
  UNIQUE(tenant_id,journal_number),
  FOREIGN KEY(tenant_id,reversal_of) REFERENCES accounting_journals(tenant_id,id),
  CHECK(status <> 'posted' OR (posted_by IS NOT NULL AND posted_at IS NOT NULL AND total_debit = total_credit AND total_debit > 0))
);
CREATE TABLE accounting_journal_lines (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL,
  journal_id uuid NOT NULL,
  account_id uuid NOT NULL,
  line_number integer NOT NULL CHECK(line_number > 0),
  description text NOT NULL DEFAULT '',
  debit numeric(20,4) NOT NULL DEFAULT 0 CHECK(debit >= 0),
  credit numeric(20,4) NOT NULL DEFAULT 0 CHECK(credit >= 0),
  FOREIGN KEY(tenant_id,journal_id) REFERENCES accounting_journals(tenant_id,id) ON DELETE RESTRICT,
  FOREIGN KEY(tenant_id,account_id) REFERENCES accounting_accounts(tenant_id,id) ON DELETE RESTRICT,
  UNIQUE(journal_id,line_number),
  CHECK((debit > 0 AND credit = 0) OR (credit > 0 AND debit = 0))
);

CREATE TABLE document_objects (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL REFERENCES tenants(id) ON DELETE RESTRICT,
  engagement_id uuid,
  workpaper_id uuid,
  report_id uuid,
  object_key text NOT NULL UNIQUE,
  original_filename text NOT NULL,
  media_type text NOT NULL,
  size_bytes bigint NOT NULL CHECK(size_bytes > 0 AND size_bytes <= 52428800),
  sha256 text NOT NULL CHECK(sha256 ~ '^[a-f0-9]{64}$'),
  scan_status text NOT NULL DEFAULT 'pending' CHECK(scan_status IN ('pending','scanning','clean','infected','failed','quarantined')),
  uploaded_by uuid NOT NULL REFERENCES users(id),
  scanned_at timestamptz,
  scan_detail text,
  created_at timestamptz NOT NULL DEFAULT now(),
  FOREIGN KEY(tenant_id,engagement_id) REFERENCES audit_engagements(tenant_id,id),
  FOREIGN KEY(tenant_id,workpaper_id) REFERENCES audit_workpapers(tenant_id,id),
  FOREIGN KEY(tenant_id,report_id) REFERENCES audit_reports(tenant_id,id),
  UNIQUE(tenant_id,id)
);
CREATE INDEX document_scan_queue_idx ON document_objects(created_at) WHERE scan_status='pending';

CREATE TABLE email_outbox (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id uuid REFERENCES users(id),
  tenant_id uuid REFERENCES tenants(id),
  recipient text NOT NULL,
  template_key text NOT NULL,
  payload jsonb NOT NULL,
  status text NOT NULL DEFAULT 'queued' CHECK(status IN ('queued','sent','failed')),
  attempts integer NOT NULL DEFAULT 0,
  available_at timestamptz NOT NULL DEFAULT now(),
  sent_at timestamptz,
  last_error text,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE audit_domain_history (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id uuid NOT NULL REFERENCES tenants(id),
  actor_id uuid NOT NULL REFERENCES users(id),
  entity_type text NOT NULL,
  entity_id uuid NOT NULL,
  action text NOT NULL,
  before_data jsonb,
  after_data jsonb,
  correlation_id text,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX audit_domain_history_entity_idx ON audit_domain_history(tenant_id,entity_type,entity_id,created_at);

-- RLS is mandatory for every tenant-owned domain and security table.
DO $arm$
DECLARE t text;
BEGIN
 FOREACH t IN ARRAY ARRAY['tenant_roles','tenant_role_permissions','tenant_membership_roles','organization_invitations','audit_engagements','audit_workpapers','audit_reports','accounting_accounts','accounting_journals','accounting_journal_lines','document_objects','email_outbox','audit_domain_history'] LOOP
   EXECUTE format('ALTER TABLE %I ENABLE ROW LEVEL SECURITY',t);
   EXECUTE format('ALTER TABLE %I FORCE ROW LEVEL SECURITY',t);
   EXECUTE format('CREATE POLICY %I ON %I USING (tenant_id::text = current_setting(''app.tenant_id'',true) AND EXISTS (SELECT 1 FROM tenant_memberships m JOIN tenants t ON t.id=m.tenant_id WHERE m.tenant_id=%I.tenant_id AND m.user_id::text=current_setting(''app.user_id'',true) AND m.status=''active'' AND t.status=''active'')) WITH CHECK (tenant_id::text=current_setting(''app.tenant_id'',true) AND EXISTS (SELECT 1 FROM tenant_memberships m JOIN tenants t ON t.id=m.tenant_id WHERE m.tenant_id=%I.tenant_id AND m.user_id::text=current_setting(''app.user_id'',true) AND m.status=''active'' AND t.status=''active''))',t||'_tenant_scope',t,t,t);
 END LOOP;
END $arm$;

ALTER TABLE email_verification_tokens ENABLE ROW LEVEL SECURITY;
ALTER TABLE email_verification_tokens FORCE ROW LEVEL SECURITY;
CREATE POLICY email_tokens_self ON email_verification_tokens USING(user_id::text=current_setting('app.user_id',true)) WITH CHECK(user_id::text=current_setting('app.user_id',true));
ALTER TABLE password_recovery_tokens ENABLE ROW LEVEL SECURITY;
ALTER TABLE password_recovery_tokens FORCE ROW LEVEL SECURITY;
CREATE POLICY recovery_tokens_self ON password_recovery_tokens USING(user_id::text=current_setting('app.user_id',true)) WITH CHECK(user_id::text=current_setting('app.user_id',true));
ALTER TABLE mfa_credentials ENABLE ROW LEVEL SECURITY;
ALTER TABLE mfa_credentials FORCE ROW LEVEL SECURITY;
CREATE POLICY mfa_self ON mfa_credentials USING(user_id::text=current_setting('app.user_id',true)) WITH CHECK(user_id::text=current_setting('app.user_id',true));
ALTER TABLE mfa_recovery_codes ENABLE ROW LEVEL SECURITY;
ALTER TABLE mfa_recovery_codes FORCE ROW LEVEL SECURITY;
CREATE POLICY mfa_recovery_self ON mfa_recovery_codes USING(user_id::text=current_setting('app.user_id',true)) WITH CHECK(user_id::text=current_setting('app.user_id',true));

-- Worker access is granted only to operational queues and document scan metadata.
CREATE POLICY email_outbox_worker_access ON email_outbox USING(current_setting('app.worker',true)='true') WITH CHECK(current_setting('app.worker',true)='true');
CREATE POLICY document_objects_worker_access ON document_objects USING(current_setting('app.worker',true)='true') WITH CHECK(current_setting('app.worker',true)='true');
GRANT SELECT,UPDATE ON email_outbox TO arm_worker;
GRANT SELECT,UPDATE ON document_objects TO arm_worker;


CREATE TABLE auth_mfa_challenges (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  challenge_hash text NOT NULL UNIQUE,
  expires_at timestamptz NOT NULL,
  consumed_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE auth_mfa_challenges ENABLE ROW LEVEL SECURITY;
ALTER TABLE auth_mfa_challenges FORCE ROW LEVEL SECURITY;
CREATE POLICY mfa_challenges_self ON auth_mfa_challenges USING(user_id::text=current_setting('app.user_id',true)) WITH CHECK(user_id::text=current_setting('app.user_id',true));

CREATE OR REPLACE FUNCTION public.arm_lookup_verification(p_hash text)
RETURNS TABLE(token_id uuid,user_id uuid,tenant_id uuid,email text)
LANGUAGE sql SECURITY DEFINER SET search_path=pg_catalog,public AS $fn$
 SELECT v.id,u.id,m.tenant_id,u.email FROM public.email_verification_tokens v
 JOIN public.users u ON u.id=v.user_id
 JOIN public.tenant_memberships m ON m.user_id=u.id AND m.status='active'
 WHERE v.token_hash=p_hash AND v.consumed_at IS NULL AND v.expires_at>now()
 ORDER BY m.created_at LIMIT 1
$fn$;
CREATE OR REPLACE FUNCTION public.arm_lookup_recovery(p_hash text)
RETURNS TABLE(token_id uuid,user_id uuid)
LANGUAGE sql SECURITY DEFINER SET search_path=pg_catalog,public AS $fn$
 SELECT r.id,u.id FROM public.password_recovery_tokens r JOIN public.users u ON u.id=r.user_id
 WHERE r.token_hash=p_hash AND r.consumed_at IS NULL AND r.expires_at>now() AND u.status='active'
$fn$;
CREATE OR REPLACE FUNCTION public.arm_lookup_account(p_email text)
RETURNS TABLE(user_id uuid,tenant_id uuid,email text)
LANGUAGE sql SECURITY DEFINER SET search_path=pg_catalog,public AS $fn$
 SELECT u.id,m.tenant_id,u.email FROM public.users u JOIN public.tenant_memberships m ON m.user_id=u.id AND m.status='active'
 WHERE u.email_normalized=lower(btrim(p_email)) AND u.status='active' ORDER BY m.created_at LIMIT 1
$fn$;
REVOKE ALL ON FUNCTION public.arm_lookup_verification(text) FROM PUBLIC;
REVOKE ALL ON FUNCTION public.arm_lookup_recovery(text) FROM PUBLIC;
REVOKE ALL ON FUNCTION public.arm_lookup_account(text) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.arm_lookup_verification(text) TO arm_runtime;
GRANT EXECUTE ON FUNCTION public.arm_lookup_recovery(text) TO arm_runtime;
GRANT EXECUTE ON FUNCTION public.arm_lookup_account(text) TO arm_runtime;

ALTER TABLE auth_mfa_challenges ENABLE ROW LEVEL SECURITY;
ALTER TABLE auth_mfa_challenges FORCE ROW LEVEL SECURITY;
CREATE POLICY auth_mfa_challenge_self ON auth_mfa_challenges USING(user_id::text=current_setting('app.user_id',true)) WITH CHECK(user_id::text=current_setting('app.user_id',true));

DO $arm$
BEGIN
 IF EXISTS(SELECT 1 FROM pg_roles WHERE rolname='arm_runtime') THEN
  GRANT SELECT,INSERT,UPDATE ON auth_mfa_challenges TO arm_runtime;
 END IF;
END $arm$;

-- Append-only accounting postings and domain history.
CREATE OR REPLACE FUNCTION arm_prevent_posted_journal_mutation() RETURNS trigger LANGUAGE plpgsql AS $fn$
BEGIN
 IF OLD.status IN ('posted','reversed') THEN RAISE EXCEPTION 'posted journal is immutable; use a reversal' USING ERRCODE='55000'; END IF;
 RETURN NEW;
END $fn$;
CREATE TRIGGER posted_journal_guard BEFORE UPDATE OR DELETE ON accounting_journals FOR EACH ROW EXECUTE FUNCTION arm_prevent_posted_journal_mutation();
CREATE OR REPLACE FUNCTION arm_prevent_posted_line_mutation() RETURNS trigger LANGUAGE plpgsql AS $fn$
DECLARE s text;
BEGIN
 SELECT status INTO s FROM accounting_journals WHERE id=COALESCE(OLD.journal_id,NEW.journal_id);
 IF s IN ('posted','reversed') THEN RAISE EXCEPTION 'posted journal lines are immutable' USING ERRCODE='55000'; END IF;
 IF TG_OP='DELETE' THEN RETURN OLD; END IF;
 RETURN NEW;
END $fn$;
CREATE TRIGGER posted_journal_line_guard BEFORE INSERT OR UPDATE OR DELETE ON accounting_journal_lines FOR EACH ROW EXECUTE FUNCTION arm_prevent_posted_line_mutation();

-- API runtime is deliberately granted only the v2 tables it needs. No blanket default grants.
DO $arm$
BEGIN
 IF EXISTS(SELECT 1 FROM pg_roles WHERE rolname='arm_runtime') THEN
  GRANT SELECT,INSERT,UPDATE ON email_verification_tokens,password_recovery_tokens,mfa_credentials,mfa_recovery_codes TO arm_runtime;
  GRANT SELECT,INSERT,UPDATE ON tenant_roles,tenant_role_permissions,tenant_membership_roles,organization_invitations TO arm_runtime;
  GRANT SELECT,INSERT,UPDATE ON audit_engagements,audit_workpapers,audit_reports,accounting_accounts,accounting_journals,accounting_journal_lines TO arm_runtime;
  GRANT SELECT,INSERT,UPDATE ON document_objects,email_outbox,audit_domain_history TO arm_runtime;
 END IF;
END $arm$;
