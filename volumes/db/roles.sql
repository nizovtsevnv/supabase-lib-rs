-- Custom roles for local development

-- Grant additional permissions to anon role
grant usage on schema public to anon;
grant usage on schema storage to anon;
grant all on all tables in schema public to anon;
grant all on all tables in schema storage to anon;

-- Grant full access to service_role
grant all privileges on database postgres to service_role;
grant all on schema public to service_role;
grant all on schema storage to service_role;
grant all on all tables in schema public to service_role;
grant all on all tables in schema storage to service_role;

-- Create test table for integration tests
create table if not exists public.test_table (
  id serial primary key,
  name text not null,
  description text,
  created_at timestamp with time zone default now()
);

-- Allow anon to access test table
grant all on table public.test_table to anon;
grant all on sequence public.test_table_id_seq to anon;
