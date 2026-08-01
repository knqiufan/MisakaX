mod context;
mod git;
mod types;

pub use context::WorkspaceContextService;
pub use git::GitCliProvider;
pub(crate) use types::strip_windows_verbatim_prefix;
pub use types::{
    CanonicalWorkspace, DetectionCancellation, VcsContext, VcsDiagnostic, VcsProvider,
};
