use std::path::PathBuf;

use crate::config::Config;

use super::process::calling_process;

pub fn absolute_path(relative_path: &str, config: &Config) -> Option<PathBuf> {
    match (
        &config.cwd_of_delta_process,
        &config.cwd_of_user_shell_process,
        calling_process().is_git_diff_relative() || config.relative_paths,
    ) {
        // Note that if we were invoked by git then cwd_of_delta_process == repo_root
        (Some(cwd_of_delta_process), _, false) => Some(cwd_of_delta_process.join(relative_path)),
        (_, Some(cwd_of_user_shell_process), true) => {
            Some(cwd_of_user_shell_process.join(relative_path))
        }
        (Some(cwd_of_delta_process), None, true) => {
            // This might occur when piping from git to delta?
            Some(cwd_of_delta_process.join(relative_path))
        }
        _ => None,
    }
}

/// Relativize path if delta config demands that and paths are not already relativized by git.
pub fn relativize_path_maybe(path: &str, config: &Config) -> Option<PathBuf> {
    if config.relative_paths && !calling_process().is_git_diff_relative() {
        if let Some(base) = config.cwd_relative_to_repo_root.as_deref() {
            pathdiff::diff_paths(&path, base)
        } else {
            None
        }
    } else {
        None
    }
}

/// Return current working directory of the user's shell process. I.e. the directory which they are
/// in when delta exits. This is the directory relative to which the file paths in delta output are
/// constructed if they are using either (a) delta's relative-paths option or (b) git's --relative
/// flag.
pub fn cwd_of_user_shell_process(
    cwd_of_delta_process: Option<&PathBuf>,
    cwd_relative_to_repo_root: Option<&str>,
) -> Option<PathBuf> {
    match (cwd_of_delta_process, cwd_relative_to_repo_root) {
        (Some(cwd), None) => {
            // We are not a child process of git
            Some(PathBuf::from(cwd))
        }
        (Some(repo_root), Some(cwd_relative_to_repo_root)) => {
            // We are a child process of git; git spawned us from repo_root and preserved the user's
            // original cwd in the GIT_PREFIX env var (available as config.cwd_relative_to_repo_root)
            Some(PathBuf::from(repo_root).join(cwd_relative_to_repo_root))
        }
        (None, _) => {
            // Unexpected
            None
        }
    }
}
