use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Hash, Debug, Serialize, Deserialize, Error)]
pub enum ValidateSubmitError {
    #[error("invalid assignment")]
    InvalidAssignment,
    #[error("task already validated and this result is not valid")]
    InconsistentValidationState,
    #[error("all results are inconclusive, and no new assignment has finished to solve it")]
    ValidationImpossibleError,
    #[error("validation group contained assignments belonging to multiple tasks")]
    ValidationGroupTaskInconsistency,
    #[error("assignments referred to by group id cannot refer to an assignment other than itself")]
    ValidationGroupAssociationInconsistency,
    #[error("state transition forbidden")]
    StateTransitionForbidden,
}
