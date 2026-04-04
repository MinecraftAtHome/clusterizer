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
    created_by_user_id int8 NOT NULL REFERENCES users(id) ON DELETE RESTRICT ON UPDATE RESTRICT,
    disabled_at timestamptz,
    name text NOT NULL
);

CREATE TABLE files (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    url text NOT NULL,
    hash bytea NOT NULL
);

CREATE TABLE platforms (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    name text NOT NULL,
    file_id int8 NOT NULL REFERENCES files(id) ON DELETE RESTRICT ON UPDATE RESTRICT
);

CREATE TABLE project_versions (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    disabled_at timestamptz,
    project_id int8 NOT NULL REFERENCES projects(id) ON DELETE CASCADE ON UPDATE CASCADE,
    platform_id int8 NOT NULL REFERENCES platforms(id) ON DELETE RESTRICT ON UPDATE RESTRICT,
    file_id int8 NOT NULL REFERENCES files(id) ON DELETE RESTRICT ON UPDATE RESTRICT
);

CREATE TABLE tasks (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    deadline interval NOT NULL,
    project_id int8 NOT NULL REFERENCES projects(id) ON DELETE CASCADE ON UPDATE CASCADE,
    stdin text NOT NULL,
    assignments_needed int4 NOT NULL,
    assignment_user_ids int8[] NOT NULL DEFAULT ARRAY[]::int8[],
    quorum int4 NOT NULL
);

CREATE TYPE assignment_state AS ENUM (
    'init',
    'canceled',
    'expired',
    'submitted'
);

CREATE TABLE assignments (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    deadline_at timestamptz NOT NULL,
    task_id int8 NOT NULL REFERENCES tasks(id) ON DELETE CASCADE ON UPDATE CASCADE,
    user_id int8 NOT NULL REFERENCES users(id) ON DELETE RESTRICT ON UPDATE RESTRICT,
    state assignment_state NOT NULL DEFAULT 'init'
);

CREATE UNIQUE INDEX assignments_task_id_user_id_key
ON assignments (task_id, user_id)
WHERE state != 'canceled' AND state != 'expired';

CREATE TYPE result_state AS ENUM (
    'init',
    'valid',
    'invalid',
    'inconclusive',
    'error'
);

CREATE TABLE results (
    id int8 GENERATED ALWAYS AS IDENTITY NOT NULL PRIMARY KEY,
    created_at timestamptz NOT NULL DEFAULT now(),
    state result_state NOT NULL DEFAULT 'init',
    assignment_id int8 NOT NULL UNIQUE REFERENCES assignments(id) ON DELETE CASCADE ON UPDATE CASCADE,
    stdout text NOT NULL,
    stderr text NOT NULL,
    exit_code int4,
    group_result_id int8 REFERENCES results(id) ON DELETE RESTRICT ON UPDATE RESTRICT
);

-- update assignments.state
CREATE FUNCTION set_assignments_state_submitted()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$ BEGIN
    UPDATE
        assignments
    SET
        state = 'submitted'
    WHERE
        id = NEW.assignment_id;

    RETURN NEW;
END $$;

CREATE TRIGGER set_assignments_state_submitted_before_insert
BEFORE INSERT
ON results
FOR EACH ROW
EXECUTE FUNCTION set_assignments_state_submitted();

-- update tasks.assignment_user_ids
CREATE FUNCTION remove_tasks_assignment_user_id()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$ BEGIN
    UPDATE
        tasks
    SET
        assignment_user_ids = array_remove(assignment_user_ids, OLD.user_id)
    WHERE
        id = OLD.task_id;

    RETURN NEW;
END $$;

CREATE FUNCTION add_tasks_assignment_user_id()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$ BEGIN
    UPDATE
        tasks
    SET
        assignment_user_ids = assignment_user_ids || NEW.user_id
    WHERE
        id = NEW.task_id;

    RETURN NEW;
END $$;

CREATE TRIGGER remove_tasks_assignment_user_id_before_update
BEFORE UPDATE OF state
ON assignments
FOR EACH ROW
WHEN ((OLD.state = 'init' OR OLD.state = 'submitted') AND (NEW.state = 'canceled' OR NEW.state = 'expired'))
EXECUTE FUNCTION remove_tasks_assignment_user_id();

CREATE TRIGGER remove_tasks_assignment_user_id_before_delete
BEFORE DELETE
ON assignments
FOR EACH ROW
WHEN (OLD.state = 'init' OR OLD.state = 'submitted')
EXECUTE FUNCTION remove_tasks_assignment_user_id();

CREATE TRIGGER add_tasks_assignment_user_id_before_insert
BEFORE INSERT
ON assignments
FOR EACH ROW
WHEN (NEW.state = 'init' OR NEW.state = 'submitted')
EXECUTE FUNCTION add_tasks_assignment_user_id();

-- set tasks.assignments_needed
CREATE FUNCTION set_tasks_assignments_needed()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$ BEGIN
    NEW.assignments_needed := NEW.quorum;

    RETURN NEW;
END $$;

CREATE TRIGGER set_tasks_assignments_needed_before_insert
BEFORE INSERT
ON tasks
FOR EACH ROW
EXECUTE FUNCTION set_tasks_assignments_needed();

-- set assignments.deadline_at
CREATE FUNCTION set_assignments_deadline_at()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$ BEGIN
    SELECT now() + deadline FROM tasks WHERE id = NEW.task_id INTO NEW.deadline_at;

    RETURN NEW;
END $$;

CREATE TRIGGER set_assignments_deadline_at_before_insert
BEFORE INSERT
ON assignments
FOR EACH ROW
EXECUTE FUNCTION set_assignments_deadline_at();
