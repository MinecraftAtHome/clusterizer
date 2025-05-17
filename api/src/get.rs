use clusterizer_common::records::{
    Assignment, AssignmentFilter, Platform, PlatformFilter, Project, ProjectFilter, ProjectVersion,
    ProjectVersionFilter, Result, ResultFilter, Task, TaskFilter, User, UserFilter,
};

pub trait Get {
    type Filter;

    const PATH: &str;
}

impl Get for User {
    type Filter = UserFilter;

    const PATH: &str = "users";
}

impl Get for Project {
    type Filter = ProjectFilter;

    const PATH: &str = "projects";
}

impl Get for Platform {
    type Filter = PlatformFilter;

    const PATH: &str = "platforms";
}

impl Get for ProjectVersion {
    type Filter = ProjectVersionFilter;

    const PATH: &str = "project_versions";
}

impl Get for Task {
    type Filter = TaskFilter;

    const PATH: &str = "tasks";
}

impl Get for Assignment {
    type Filter = AssignmentFilter;

    const PATH: &str = "assignments";
}

impl Get for Result {
    type Filter = ResultFilter;

    const PATH: &str = "results";
}
