pub mod fetch_tasks_error;
pub mod infallible;
pub mod not_found;
pub mod register_error;
pub mod submit_result_error;
pub mod validate_error;

pub use fetch_tasks_error::FetchTasksError;
pub use infallible::Infallible;
pub use not_found::NotFound;
pub use register_error::RegisterError;
pub use submit_result_error::SubmitResultError;
pub use validate_error::ValidateFetchError;
pub use validate_error::ValidateOkError;
