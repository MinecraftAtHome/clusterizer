use clusterizer_common::{
    records::{
        Assignment, AssignmentFilter, File, FileFilter, Platform, PlatformFilter, Project,
        ProjectFilter, ProjectVersion, ProjectVersionFilter, Result, ResultFilter, Task,
        TaskFilter, User, UserFilter,
    },
    types::Id,
};

use super::Map;

pub trait Select: Sized {
    type Filter;

    fn select_all(filter: &Self::Filter) -> Map<Self>;
    fn select_one(id: Id<Self>) -> Map<Self>;
}

impl Select for User {
    type Filter = UserFilter;

    fn select_all(filter: &Self::Filter) -> Map<Self> {
        sqlx::query_as_unchecked!(
            Self,
            r#"
            SELECT
                *
            FROM
                users
            WHERE
                disabled_at IS NULL IS DISTINCT FROM $1
            "#,
            filter.disabled,
        )
    }

    fn select_one(id: Id<Self>) -> Map<Self> {
        sqlx::query_as_unchecked!(Self, "SELECT * FROM users WHERE id = $1", id)
    }
}

impl Select for Project {
    type Filter = ProjectFilter;

    fn select_all(filter: &Self::Filter) -> Map<Self> {
        sqlx::query_as_unchecked!(
            Self,
            r#"
            SELECT
                *
            FROM
                projects
            WHERE
                created_by_user_id = $1 IS NOT FALSE
                AND disabled_at IS NULL IS DISTINCT FROM $2
            "#,
            filter.created_by_user_id,
            filter.disabled,
        )
    }

    fn select_one(id: Id<Self>) -> Map<Self> {
        sqlx::query_as_unchecked!(Self, "SELECT * FROM projects WHERE id = $1", id)
    }
}

impl Select for Platform {
    type Filter = PlatformFilter;

    fn select_all(filter: &Self::Filter) -> Map<Self> {
        sqlx::query_as_unchecked!(
            Self,
            r#"
            SELECT
                *
            FROM
                platforms
            WHERE
                file_id = $1 IS NOT FALSE
            "#,
            filter.file_id,
        )
    }

    fn select_one(id: Id<Self>) -> Map<Self> {
        sqlx::query_as_unchecked!(Self, "SELECT * FROM platforms WHERE id = $1", id)
    }
}

impl Select for ProjectVersion {
    type Filter = ProjectVersionFilter;

    fn select_all(filter: &Self::Filter) -> Map<Self> {
        sqlx::query_as_unchecked!(
            Self,
            r#"
            SELECT
                *
            FROM
                project_versions
            WHERE
                disabled_at IS NULL IS DISTINCT FROM $1
                AND project_id = $2 IS NOT FALSE
                AND platform_id = $3 IS NOT FALSE
                AND file_id = $4 IS NOT FALSE
            "#,
            filter.disabled,
            filter.project_id,
            filter.platform_id,
            filter.file_id,
        )
    }

    fn select_one(id: Id<Self>) -> Map<Self> {
        sqlx::query_as_unchecked!(Self, "SELECT * FROM project_versions WHERE id = $1", id)
    }
}

impl Select for File {
    type Filter = FileFilter;

    fn select_all(_: &Self::Filter) -> Map<Self> {
        sqlx::query_as_unchecked!(
            Self,
            r#"
            SELECT
                *
            FROM
                files
            "#,
        )
    }

    fn select_one(id: Id<Self>) -> Map<Self> {
        sqlx::query_as_unchecked!(Self, "SELECT * FROM files WHERE id = $1", id)
    }
}

impl Select for Task {
    type Filter = TaskFilter;

    fn select_all(filter: &Self::Filter) -> Map<Self> {
        sqlx::query_as_unchecked!(
            Self,
            r#"
            SELECT
                *
            FROM
                tasks
            WHERE
                project_id = $1 IS NOT FALSE
            "#,
            filter.project_id,
        )
    }

    fn select_one(id: Id<Self>) -> Map<Self> {
        sqlx::query_as_unchecked!(Self, "SELECT * FROM tasks WHERE id = $1", id)
    }
}

impl Select for Assignment {
    type Filter = AssignmentFilter;

    fn select_all(filter: &Self::Filter) -> Map<Self> {
        sqlx::query_as_unchecked!(
            Self,
            r#"
            SELECT
                *
            FROM
                assignments
            WHERE
                task_id = $1 IS NOT FALSE
                AND user_id = $2 IS NOT FALSE
                AND state = $3 IS NOT FALSE
            "#,
            filter.task_id,
            filter.user_id,
            filter.state,
        )
    }

    fn select_one(id: Id<Self>) -> Map<Self> {
        sqlx::query_as_unchecked!(Self, "SELECT * FROM assignments WHERE id = $1", id)
    }
}

impl Select for Result {
    type Filter = ResultFilter;

    fn select_all(filter: &Self::Filter) -> Map<Self> {
        sqlx::query_as_unchecked!(
            Self,
            r#"
            SELECT
                *
            FROM
                results
            WHERE
                assignment_id = $1 IS NOT FALSE
                AND (group_result_id = $2 OR $2 IS NULL)
                AND state = $3 IS NOT FALSE
            "#,
            filter.assignment_id,
            filter.group_result_id,
            filter.state,
        )
    }

    fn select_one(id: Id<Self>) -> Map<Self> {
        sqlx::query_as_unchecked!(Self, "SELECT * FROM results WHERE id = $1", id)
    }
}
