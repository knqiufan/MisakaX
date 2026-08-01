mod context;
mod git;
mod types;

pub use context::WorkspaceContextService;
pub use git::GitCliProvider;
pub use types::{
    CanonicalWorkspace, DetectionCancellation, VcsContext, VcsDiagnostic, VcsProvider,
};
