CREATE TABLE users (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    name text NOT NULL
);

CREATE UNIQUE INDEX users_name_key
    ON users (lower(name));

CREATE TABLE projects (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    name text NOT NULL,
    active bool NOT NULL
);

CREATE TABLE platforms (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    name text NOT NULL
);

CREATE TABLE project_versions (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    project_id int8 NOT NULL REFERENCES projects(id) ON DELETE CASCADE ON UPDATE CASCADE,
    platform_id int8 NOT NULL REFERENCES platforms(id) ON DELETE RESTRICT ON UPDATE RESTRICT,
    archive_url text NOT NULL
);

CREATE TABLE tasks (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    project_id int8 NOT NULL REFERENCES projects(id) ON DELETE RESTRICT ON UPDATE RESTRICT,
    stdin text NOT NULL,
    assignments_needed int4 NOT NULL DEFAULT 1,
    assigned_to_userids int8[] NOT NULL DEFAULT ARRAY[]::int8[]
);

CREATE TABLE assignments (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    task_id int8 NOT NULL REFERENCES tasks(id) ON DELETE CASCADE ON UPDATE CASCADE,
    user_id int8 NOT NULL REFERENCES users(id) ON DELETE RESTRICT ON UPDATE RESTRICT,
    canceled_at timestamptz
);

CREATE UNIQUE INDEX assignments_task_id_user_id_key
    ON assignments (task_id, user_id)
    WHERE canceled_at IS NULL;

CREATE INDEX tasks_assignments_needed_assigned_to_idx
ON tasks USING GIN (assigned_to_userids)
WHERE assignments_needed < array_length(assigned_to_userids, 1);

CREATE TABLE results (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    assignment_id int8 NOT NULL UNIQUE REFERENCES assignments(id) ON DELETE RESTRICT ON UPDATE RESTRICT,
    stdout text NOT NULL,
    stderr text NOT NULL,
    exit_code int4
);
