use clusterizer_common::{
    records::{Assignment, Platform, Project, ProjectVersion, Result, Task, User},
    types::Id,
};

pub trait GetAll: Sized {
    fn get_all(url: &str) -> String;
}

pub trait GetAllBy<T>: Sized {
    fn get_all_by(url: &str, id: Id<T>) -> String;
}

pub trait GetOne: Sized {
    fn get_one(url: &str, id: Id<Self>) -> String;
}

pub trait GetOneBy<T>: Sized {
    fn get_one_by(url: &str, id: Id<T>) -> String;
}

macro_rules! get_all {
    ($t:ty: $path:literal) => {
        impl GetAll for $t {
            fn get_all(url: &str) -> String {
                format!($path, url)
            }
        }
    };
    ($t:ty: $path:literal, $u:ty) => {
        impl GetAllBy<$u> for $t {
            fn get_all_by(url: &str, id: Id<$u>) -> String {
                format!($path, url, id)
            }
        }
    };
}

macro_rules! get_one {
    ($t:ty: $path:literal) => {
        impl GetOne for $t {
            fn get_one(url: &str, id: Id<Self>) -> String {
                format!($path, url, id)
            }
        }
    };
    ($t:ty: $path:literal, $u:ty) => {
        impl GetOneBy<$u> for $t {
            fn get_one_by(url: &str, id: Id<$u>) -> String {
                format!($path, url, id)
            }
        }
    };
}

get_all!(User: "{}/users");
get_one!(User: "{}/users/{}");
get_all!(Project: "{}/projects");
get_one!(Project: "{}/projects/{}");
get_all!(Platform: "{}/platforms");
get_one!(Platform: "{}/platforms/{}");
get_all!(ProjectVersion: "{}/project_versions");
get_one!(ProjectVersion: "{}/project_versions/{}");
get_all!(ProjectVersion: "{}/projects/{}/project_versions", Project);
get_all!(ProjectVersion: "{}/platforms/{}/project_versions", Platform);
get_all!(Task: "{}/tasks");
get_one!(Task: "{}/tasks/{}");
get_all!(Task: "{}/projects/{}/tasks", Project);
get_all!(Assignment: "{}/assignments");
get_one!(Assignment: "{}/assignments/{}");
get_all!(Assignment: "{}/users/{}/assignments", User);
get_all!(Assignment: "{}/tasks/{}/assignments", Task);
get_all!(Result: "{}/results");
get_one!(Result: "{}/results/{}");
get_one!(Result: "{}/assignments/{}/result", Assignment);
