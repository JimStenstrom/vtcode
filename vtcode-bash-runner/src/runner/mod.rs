//! Command runner for bash operations

mod paths;
mod fileops;
mod search;
mod formatting;

#[cfg(test)]
mod tests;

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

use crate::executor::{CommandCategory, CommandExecutor, CommandInvocation, ShellKind};
use crate::policy::CommandPolicy;
use vtcode_commons::WorkspacePaths;

/// High-level bash command runner
pub struct BashRunner<E, P> {
    executor: E,
    policy: P,
    workspace_root: PathBuf,
    working_dir: PathBuf,
    shell_kind: ShellKind,
}

impl<E, P> BashRunner<E, P>
where
    E: CommandExecutor,
    P: CommandPolicy,
{
    pub fn new(workspace_root: PathBuf, executor: E, policy: P) -> Result<Self> {
        if !workspace_root.exists() {
            bail!(
                "workspace root `{}` does not exist",
                workspace_root.display()
            );
        }

        let canonical_root = workspace_root
            .canonicalize()
            .with_context(|| format!("failed to canonicalize `{}`", workspace_root.display()))?;

        Ok(Self {
            executor,
            policy,
            workspace_root: canonical_root.clone(),
            working_dir: canonical_root,
            shell_kind: default_shell_kind(),
        })
    }

    pub fn from_workspace_paths<W>(paths: &W, executor: E, policy: P) -> Result<Self>
    where
        W: WorkspacePaths,
    {
        Self::new(paths.workspace_root().to_path_buf(), executor, policy)
    }

    // Accessors
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    pub fn working_dir(&self) -> &Path {
        &self.working_dir
    }

    pub fn shell_kind(&self) -> ShellKind {
        self.shell_kind
    }

    // File operations - delegate to fileops module
    pub fn ls(&self, path: Option<&str>, show_hidden: bool) -> Result<String> {
        fileops::ls(
            &self.executor,
            &self.policy,
            &self.workspace_root,
            &self.working_dir,
            self.shell_kind,
            path,
            show_hidden,
        )
    }

    pub fn pwd(&self) -> Result<String> {
        fileops::pwd(&self.policy, &self.working_dir, self.shell_kind)
    }

    pub fn mkdir(&self, path: &str, parents: bool) -> Result<()> {
        fileops::mkdir(
            &self.executor,
            &self.policy,
            &self.workspace_root,
            &self.working_dir,
            self.shell_kind,
            path,
            parents,
        )
    }

    pub fn rm(&self, path: &str, recursive: bool, force: bool) -> Result<()> {
        fileops::rm(
            &self.executor,
            &self.policy,
            &self.workspace_root,
            &self.working_dir,
            self.shell_kind,
            path,
            recursive,
            force,
        )
    }

    pub fn cp(&self, source: &str, dest: &str, recursive: bool) -> Result<()> {
        fileops::cp(
            &self.executor,
            &self.policy,
            &self.workspace_root,
            &self.working_dir,
            self.shell_kind,
            source,
            dest,
            recursive,
        )
    }

    pub fn mv(&self, source: &str, dest: &str) -> Result<()> {
        fileops::mv(
            &self.executor,
            &self.policy,
            &self.workspace_root,
            &self.working_dir,
            self.shell_kind,
            source,
            dest,
        )
    }

    // Search operations - delegate to search module
    pub fn grep(&self, pattern: &str, path: Option<&str>, recursive: bool) -> Result<String> {
        search::grep(
            &self.executor,
            &self.policy,
            &self.workspace_root,
            &self.working_dir,
            self.shell_kind,
            pattern,
            path,
            recursive,
        )
    }

    // Directory operations - implement directly using paths module
    pub fn cd(&mut self, path: &str) -> Result<()> {
        let candidate = paths::resolve_path(&self.working_dir, path);
        if !candidate.exists() {
            bail!("directory `{}` does not exist", candidate.display());
        }
        if !candidate.is_dir() {
            bail!("path `{}` is not a directory", candidate.display());
        }

        let canonical = candidate
            .canonicalize()
            .with_context(|| format!("failed to canonicalize `{}`", candidate.display()))?;

        paths::ensure_within_workspace(&self.workspace_root, &canonical)?;

        let invocation = CommandInvocation::new(
            self.shell_kind,
            format!("cd {}", formatting::format_path(self.shell_kind, &canonical)),
            CommandCategory::ChangeDirectory,
            canonical.clone(),
        )
        .with_paths(vec![canonical.clone()]);

        self.policy.check(&invocation)?;
        self.working_dir = canonical;
        Ok(())
    }
}

fn default_shell_kind() -> ShellKind {
    if cfg!(windows) {
        ShellKind::Windows
    } else {
        ShellKind::Unix
    }
}
