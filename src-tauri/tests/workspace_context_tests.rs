use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};

use misaka_x_lib::services::workspace::{
    CanonicalWorkspace, DetectionCancellation, GitCliProvider, VcsProvider, WorkspaceContextService,
};
use tempfile::TempDir;

fn git(cwd: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("git should be installed for workspace context integration tests");
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn init_repository(root: &Path) {
    std::fs::create_dir_all(root).unwrap();
    git(root, &["init"]);
    git(
        root,
        &["config", "user.email", "workspace-test@example.invalid"],
    );
    git(root, &["config", "user.name", "Workspace Test"]);
    std::fs::write(root.join("README.md"), "workspace context\n").unwrap();
    git(root, &["add", "README.md"]);
    git(root, &["commit", "-m", "initial"]);
    git(root, &["branch", "-M", "main"]);
}

async fn detect(root: &Path) -> Option<misaka_x_lib::services::workspace::VcsContext> {
    let workspace = CanonicalWorkspace::new(root).unwrap();
    GitCliProvider::default()
        .detect(&workspace, Default::default())
        .await
        .unwrap()
}

#[tokio::test]
async fn detects_branch_detached_head_and_unicode_space_paths() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("repo space-项目");
    init_repository(&root);

    let branch = detect(&root).await.unwrap();
    assert_eq!(branch.branch.as_deref(), Some("main"));
    assert!(branch.detached_head.is_none());
    assert_eq!(
        std::fs::canonicalize(&branch.repository_root).unwrap(),
        std::fs::canonicalize(&root).unwrap()
    );

    git(&root, &["checkout", "--detach", "HEAD"]);
    let detached = detect(&root).await.unwrap();
    assert!(detached.branch.is_none());
    assert!(detached
        .detached_head
        .as_ref()
        .is_some_and(|sha| sha.len() >= 7));
}

#[tokio::test]
async fn resolves_worktree_and_submodule_git_directories() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("source repo");
    init_repository(&source);
    git(&source, &["branch", "feature"]);
    let worktree = temp.path().join("feature worktree");
    git(
        &source,
        &["worktree", "add", worktree.to_str().unwrap(), "feature"],
    );

    let detected_worktree = detect(&worktree).await.unwrap();
    assert_eq!(detected_worktree.branch.as_deref(), Some("feature"));
    assert_ne!(detected_worktree.git_dir, detected_worktree.git_common_dir);

    let parent = temp.path().join("parent 项目");
    init_repository(&parent);
    let child = temp.path().join("child module");
    init_repository(&child);
    git(
        &parent,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            child.to_str().unwrap(),
            "modules/sub",
        ],
    );
    let submodule = parent.join("modules/sub");
    let detected_submodule = detect(&submodule).await.unwrap();
    assert_eq!(
        std::fs::canonicalize(&detected_submodule.repository_root).unwrap(),
        std::fs::canonicalize(&submodule).unwrap()
    );
    assert!(detected_submodule
        .git_dir
        .to_string_lossy()
        .contains("modules"));
}

#[tokio::test]
async fn bare_and_non_repository_directories_degrade_to_local() {
    let temp = TempDir::new().unwrap();
    let ordinary = temp.path().join("ordinary");
    std::fs::create_dir_all(&ordinary).unwrap();
    assert!(detect(&ordinary).await.is_none());

    let bare = temp.path().join("bare.git");
    std::fs::create_dir_all(&bare).unwrap();
    git(&bare, &["init", "--bare"]);
    assert!(detect(&bare).await.is_none());
}

#[tokio::test]
async fn cached_workspace_context_p95_stays_below_ui_budget() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("performance repo");
    init_repository(&root);
    let workspace = CanonicalWorkspace::new(&root).unwrap();
    let service = Arc::new(WorkspaceContextService::new(Arc::new(
        GitCliProvider::default(),
    )));
    let cold = service
        .get_context("performance-session", workspace.clone(), false)
        .await;
    assert_eq!(cold.branch.as_deref(), Some("main"));

    let mut samples = Vec::new();
    for _ in 0..30 {
        let started = Instant::now();
        let cached = service
            .get_context("performance-session", workspace.clone(), false)
            .await;
        assert_eq!(cached.generation, cold.generation);
        samples.push(started.elapsed());
    }
    samples.sort_unstable();
    let p95 = samples[(samples.len() * 95 / 100).min(samples.len() - 1)];
    assert!(p95 < Duration::from_millis(300), "cached P95 was {p95:?}");
}

#[tokio::test]
async fn timeout_and_missing_cli_fail_closed_without_blocking_the_caller() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("timeout repo");
    init_repository(&root);
    let workspace = CanonicalWorkspace::new(&root).unwrap();

    let started = Instant::now();
    let timeout = GitCliProvider::default()
        .with_timeout(Duration::ZERO)
        .detect(&workspace, Default::default())
        .await
        .unwrap_err();
    assert_eq!(timeout.internal_reason, "git_query_timeout");
    assert!(started.elapsed() < Duration::from_millis(300));

    let missing = GitCliProvider::with_executable(root.join("missing-git.exe"))
        .detect(&workspace, Default::default())
        .await
        .unwrap_err();
    assert_eq!(missing.internal_reason, "trusted_git_not_found");

    let cancellation = DetectionCancellation::default();
    cancellation.cancel();
    let cancelled = GitCliProvider::default()
        .detect(&workspace, cancellation)
        .await
        .unwrap_err();
    assert_eq!(cancelled.internal_reason, "git_query_cancelled");
}
