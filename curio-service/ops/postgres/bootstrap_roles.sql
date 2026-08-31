-- Run once as the Railway database administrator. Supply every password with a
-- psql variable; never add a credential to this file or enable command echoing.
--
-- Required variables:
--   curio_prod_migrator_password
--   curio_prod_app_password
--   curio_dev_migrator_password
--   curio_dev_app_password
--   curio_test_runner_password

\set ON_ERROR_STOP on

\if :{?curio_prod_migrator_password}
\else
\echo 'missing psql variable: curio_prod_migrator_password'
\quit
\endif
\if :{?curio_prod_app_password}
\else
\echo 'missing psql variable: curio_prod_app_password'
\quit
\endif
\if :{?curio_dev_migrator_password}
\else
\echo 'missing psql variable: curio_dev_migrator_password'
\quit
\endif
\if :{?curio_dev_app_password}
\else
\echo 'missing psql variable: curio_dev_app_password'
\quit
\endif
\if :{?curio_test_runner_password}
\else
\echo 'missing psql variable: curio_test_runner_password'
\quit
\endif

BEGIN;

-- Keep the shared public schema intact for catalog review, but prevent any
-- login from creating objects there merely through PUBLIC membership.
REVOKE CREATE ON SCHEMA public FROM PUBLIC;

DO $bootstrap$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'curio_prod_migrator') THEN
        CREATE ROLE curio_prod_migrator;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'curio_prod_app') THEN
        CREATE ROLE curio_prod_app;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'curio_dev_migrator') THEN
        CREATE ROLE curio_dev_migrator;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'curio_dev_app') THEN
        CREATE ROLE curio_dev_app;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'curio_test_runner') THEN
        CREATE ROLE curio_test_runner;
    END IF;
END
$bootstrap$;

-- Reassert least-privilege role attributes on every run. Passwords are safely
-- quoted by psql's :'variable' form and are not retained anywhere in the repo.
ALTER ROLE curio_prod_migrator WITH
    LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS
    PASSWORD :'curio_prod_migrator_password';
ALTER ROLE curio_prod_app WITH
    LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS
    PASSWORD :'curio_prod_app_password';
ALTER ROLE curio_dev_migrator WITH
    LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS
    PASSWORD :'curio_dev_migrator_password';
ALTER ROLE curio_dev_app WITH
    LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS
    PASSWORD :'curio_dev_app_password';
ALTER ROLE curio_test_runner WITH
    LOGIN NOINHERIT NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS
    PASSWORD :'curio_test_runner_password';

CREATE SCHEMA IF NOT EXISTS curio_prod AUTHORIZATION curio_prod_migrator;
ALTER SCHEMA curio_prod OWNER TO curio_prod_migrator;
CREATE SCHEMA IF NOT EXISTS curio_dev AUTHORIZATION curio_dev_migrator;
ALTER SCHEMA curio_dev OWNER TO curio_dev_migrator;

REVOKE ALL ON SCHEMA curio_prod FROM PUBLIC;
REVOKE ALL ON SCHEMA curio_dev FROM PUBLIC;
REVOKE ALL ON SCHEMA curio_prod FROM curio_prod_app, curio_dev_migrator, curio_dev_app, curio_test_runner;
REVOKE ALL ON SCHEMA curio_dev FROM curio_dev_app, curio_prod_migrator, curio_prod_app, curio_test_runner;
GRANT USAGE ON SCHEMA curio_prod TO curio_prod_app;
GRANT USAGE ON SCHEMA curio_dev TO curio_dev_app;

-- App roles receive data access but no schema CREATE privilege. These grants
-- cover an already-migrated schema; default privileges cover later migrations.
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA curio_prod TO curio_prod_app;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA curio_prod TO curio_prod_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA curio_dev TO curio_dev_app;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA curio_dev TO curio_dev_app;

ALTER DEFAULT PRIVILEGES FOR ROLE curio_prod_migrator IN SCHEMA curio_prod
    REVOKE ALL ON TABLES FROM PUBLIC;
ALTER DEFAULT PRIVILEGES FOR ROLE curio_prod_migrator IN SCHEMA curio_prod
    REVOKE ALL ON SEQUENCES FROM PUBLIC;
ALTER DEFAULT PRIVILEGES FOR ROLE curio_prod_migrator IN SCHEMA curio_prod
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO curio_prod_app;
ALTER DEFAULT PRIVILEGES FOR ROLE curio_prod_migrator IN SCHEMA curio_prod
    GRANT USAGE, SELECT ON SEQUENCES TO curio_prod_app;

ALTER DEFAULT PRIVILEGES FOR ROLE curio_dev_migrator IN SCHEMA curio_dev
    REVOKE ALL ON TABLES FROM PUBLIC;
ALTER DEFAULT PRIVILEGES FOR ROLE curio_dev_migrator IN SCHEMA curio_dev
    REVOKE ALL ON SEQUENCES FROM PUBLIC;
ALTER DEFAULT PRIVILEGES FOR ROLE curio_dev_migrator IN SCHEMA curio_dev
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO curio_dev_app;
ALTER DEFAULT PRIVILEGES FOR ROLE curio_dev_migrator IN SCHEMA curio_dev
    GRANT USAGE, SELECT ON SEQUENCES TO curio_dev_app;

-- Only the test runner may create disposable curio_test_<run-id> schemas. It
-- receives no grants on either long-lived Curio schema.
REVOKE CREATE ON DATABASE :"DBNAME" FROM curio_prod_migrator, curio_prod_app, curio_dev_migrator, curio_dev_app;
GRANT CONNECT ON DATABASE :"DBNAME" TO curio_prod_migrator, curio_prod_app, curio_dev_migrator, curio_dev_app, curio_test_runner;
GRANT CREATE ON DATABASE :"DBNAME" TO curio_test_runner;
REVOKE ALL PRIVILEGES ON ALL TABLES IN SCHEMA curio_prod, curio_dev FROM curio_test_runner;
REVOKE ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA curio_prod, curio_dev FROM curio_test_runner;

-- Connection code also sets these values explicitly. Role defaults provide a
-- safe fallback for administrative commands and one-off verification sessions.
ALTER ROLE curio_prod_migrator IN DATABASE :"DBNAME" SET timezone TO 'UTC';
ALTER ROLE curio_prod_migrator IN DATABASE :"DBNAME" SET search_path TO curio_prod, pg_catalog;
ALTER ROLE curio_prod_migrator IN DATABASE :"DBNAME" SET statement_timeout TO '5min';
ALTER ROLE curio_prod_migrator IN DATABASE :"DBNAME" SET idle_in_transaction_session_timeout TO '60s';

ALTER ROLE curio_prod_app IN DATABASE :"DBNAME" SET timezone TO 'UTC';
ALTER ROLE curio_prod_app IN DATABASE :"DBNAME" SET search_path TO curio_prod, pg_catalog;
ALTER ROLE curio_prod_app IN DATABASE :"DBNAME" SET statement_timeout TO '30s';
ALTER ROLE curio_prod_app IN DATABASE :"DBNAME" SET idle_in_transaction_session_timeout TO '15s';

ALTER ROLE curio_dev_migrator IN DATABASE :"DBNAME" SET timezone TO 'UTC';
ALTER ROLE curio_dev_migrator IN DATABASE :"DBNAME" SET search_path TO curio_dev, pg_catalog;
ALTER ROLE curio_dev_migrator IN DATABASE :"DBNAME" SET statement_timeout TO '5min';
ALTER ROLE curio_dev_migrator IN DATABASE :"DBNAME" SET idle_in_transaction_session_timeout TO '60s';

ALTER ROLE curio_dev_app IN DATABASE :"DBNAME" SET timezone TO 'UTC';
ALTER ROLE curio_dev_app IN DATABASE :"DBNAME" SET search_path TO curio_dev, pg_catalog;
ALTER ROLE curio_dev_app IN DATABASE :"DBNAME" SET statement_timeout TO '30s';
ALTER ROLE curio_dev_app IN DATABASE :"DBNAME" SET idle_in_transaction_session_timeout TO '15s';

-- Test fixtures set a validated per-run search_path after creating their
-- schema. pg_catalog-only is deliberately unusable until they do so.
ALTER ROLE curio_test_runner IN DATABASE :"DBNAME" SET timezone TO 'UTC';
ALTER ROLE curio_test_runner IN DATABASE :"DBNAME" SET search_path TO pg_catalog;
ALTER ROLE curio_test_runner IN DATABASE :"DBNAME" SET statement_timeout TO '60s';
ALTER ROLE curio_test_runner IN DATABASE :"DBNAME" SET idle_in_transaction_session_timeout TO '30s';

COMMIT;
