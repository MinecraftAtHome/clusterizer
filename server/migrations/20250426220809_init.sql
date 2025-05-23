CREATE TABLE users (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    disabled_at timestamptz,
    name text NOT NULL
);

CREATE UNIQUE INDEX users_name_key
    ON users (lower(name));

CREATE TABLE projects (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    disabled_at timestamptz,
    name text NOT NULL
);

CREATE TABLE platforms (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    name text NOT NULL,
    tester_archive_url text NOT NULL
);

CREATE TABLE project_versions (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    disabled_at timestamptz,
    project_id int8 NOT NULL REFERENCES projects(id) ON DELETE CASCADE ON UPDATE CASCADE,
    platform_id int8 NOT NULL REFERENCES platforms(id) ON DELETE RESTRICT ON UPDATE RESTRICT,
    archive_url text NOT NULL
);

CREATE TABLE tasks (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    project_id int8 NOT NULL REFERENCES projects(id) ON DELETE RESTRICT ON UPDATE RESTRICT,
    stdin text NOT NULL,
    assignments_needed int4 NOT NULL,
    assignment_user_ids int8[] NOT NULL DEFAULT ARRAY[]::int8[]
);

CREATE TYPE assignment_state AS ENUM (
    'init', 
    'canceled', 
    'expired',
    'submitted', 
    'valid', 
    'invalid',  
    'inconclusive'
);

CREATE TABLE assignments (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    task_id int8 NOT NULL REFERENCES tasks(id) ON DELETE CASCADE ON UPDATE CASCADE,
    user_id int8 NOT NULL REFERENCES users(id) ON DELETE RESTRICT ON UPDATE RESTRICT,
    state assignment_state NOT NULL DEFAULT 'init'
);

CREATE UNIQUE INDEX assignments_task_id_user_id_key
    ON assignments (task_id, user_id)
    WHERE state != 'canceled' AND state != 'expired';

CREATE TABLE results (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    assignment_id int8 NOT NULL UNIQUE REFERENCES assignments(id) ON DELETE RESTRICT ON UPDATE RESTRICT,
    stdout text NOT NULL,
    stderr text NOT NULL,
    exit_code int4
);
