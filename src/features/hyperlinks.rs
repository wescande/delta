use std::borrow::Cow;
use std::path::Path;
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::{Captures, Regex};

use crate::config::Config;
use crate::features::OptionValueFunction;
use crate::git_config::{GitConfig, GitConfigEntry, GitRemoteRepo};

pub fn make_feature() -> Vec<(String, OptionValueFunction)> {
    builtin_feature!([
        (
            "hyperlinks",
            bool,
            None,
            _opt => true
        )
    ])
}

pub fn format_commit_line_with_osc8_commit_hyperlink<'a>(
    line: &'a str,
    config: &Config,
) -> Cow<'a, str> {
    if let Some(commit_link_format) = &config.hyperlinks_commit_link_format {
        COMMIT_LINE_REGEX.replace(line, |captures: &Captures| {
            let commit = captures.get(2).unwrap().as_str();
            format_osc8_hyperlink(&commit_link_format.replace("{commit}", commit), commit)
        })
    } else if let Some(GitConfigEntry::GitRemote(GitRemoteRepo::GitHubRepo(repo))) =
        config.git_config.as_ref().and_then(get_remote_url)
    {
        COMMIT_LINE_REGEX.replace(line, |captures: &Captures| {
            format_commit_line_captures_with_osc8_commit_hyperlink(captures, &repo)
        })
    } else {
        Cow::from(line)
    }
}

fn get_remote_url(git_config: &GitConfig) -> Option<GitConfigEntry> {
    git_config
        .repo
        .as_ref()?
        .find_remote("origin")
        .ok()?
        .url()
        .and_then(|url| {
            GitRemoteRepo::from_str(url)
                .ok()
                .map(GitConfigEntry::GitRemote)
        })
}

/// Create a file hyperlink, displaying `text`.
pub fn format_osc8_file_hyperlink<'a, P>(
    absolute_path: P,
    line_number: Option<usize>,
    text: &str,
    config: &Config,
) -> Cow<'a, str>
where
    P: AsRef<Path>,
    P: std::fmt::Debug,
{
    debug_assert!(absolute_path.as_ref().is_absolute());
    let mut url = config
        .hyperlinks_file_link_format
        .replace("{path}", &absolute_path.as_ref().to_string_lossy());
    if let Some(n) = line_number {
        url = url.replace("{line}", &format!("{}", n))
    } else {
        url = url.replace("{line}", "")
    };
    Cow::from(format_osc8_hyperlink(&url, text))
}

fn format_osc8_hyperlink(url: &str, text: &str) -> String {
    format!(
        "{osc}8;;{url}{st}{text}{osc}8;;{st}",
        url = url,
        text = text,
        osc = "\x1b]",
        st = "\x1b\\"
    )
}

lazy_static! {
    static ref COMMIT_LINE_REGEX: Regex = Regex::new("(.* )?([0-9a-f]{8,40})(.*)").unwrap();
}

fn format_commit_line_captures_with_osc8_commit_hyperlink(
    captures: &Captures,
    github_repo: &str,
) -> String {
    let commit = captures.get(2).unwrap().as_str();
    format!(
        "{prefix}{osc}8;;{url}{st}{commit}{osc}8;;{st}{suffix}",
        url = format_github_commit_url(commit, github_repo),
        commit = commit,
        prefix = captures.get(1).map(|m| m.as_str()).unwrap_or(""),
        suffix = captures.get(3).unwrap().as_str(),
        osc = "\x1b]",
        st = "\x1b\\"
    )
}

fn format_github_commit_url(commit: &str, github_repo: &str) -> String {
    format!("https://github.com/{}/commit/{}", github_repo, commit)
}

#[cfg(not(target_os = "windows"))]
#[cfg(test)]
pub mod tests {
    use std::iter::FromIterator;
    use std::path::PathBuf;

    use super::*;
    use crate::{
        tests::integration_test_utils::{self, DeltaTest},
        utils,
    };

    #[test]
    fn test_paths_and_hyperlinks_user_in_repo_root_dir() {
        // Expectations are uninfluenced by git's --relative and delta's relative_paths options.
        let input_type = InputType::GitDiff;
        let file_path_relative_to_repo_root = PathBuf::from("a");
        let cwd_relative_to_repo_root = "";

        for (delta_relative_paths, calling_cmd) in vec![
            (false, Some("git diff")),
            (false, Some("git diff --relative")),
            (true, Some("git diff")),
            (true, Some("git diff --relative")),
        ] {
            run_test(FilePathsTestCase {
                name: &format!(
                    "delta relative_paths={} calling_cmd={:?}",
                    delta_relative_paths, calling_cmd
                ),
                file_path_relative_to_repo_root: file_path_relative_to_repo_root.as_path(),
                cwd_relative_to_repo_root,
                delta_relative_paths,
                input_type,
                calling_cmd,
                expected_displayed_path: "a",
            })
        }
    }

    #[test]
    fn test_paths_and_hyperlinks_user_in_subdir_file_in_same_subdir() {
        let input_type = InputType::GitDiff;
        let file_path_relative_to_repo_root = PathBuf::from_iter(&["b", "a"]);
        let cwd_relative_to_repo_root = "b";

        run_test(FilePathsTestCase {
            name: "b/a from b",
            file_path_relative_to_repo_root: file_path_relative_to_repo_root.as_path(),
            cwd_relative_to_repo_root,
            delta_relative_paths: false,
            input_type,
            calling_cmd: Some("git diff"),
            expected_displayed_path: "b/a",
        });
        run_test(FilePathsTestCase {
            name: "b/a from b",
            file_path_relative_to_repo_root: file_path_relative_to_repo_root.as_path(),
            cwd_relative_to_repo_root,
            delta_relative_paths: false,
            input_type,
            calling_cmd: Some("git diff --relative"),
            // delta saw a and wasn't configured to make any changes
            expected_displayed_path: "a",
        });
        run_test(FilePathsTestCase {
            name: "b/a from b",
            file_path_relative_to_repo_root: file_path_relative_to_repo_root.as_path(),
            cwd_relative_to_repo_root,
            delta_relative_paths: true,
            input_type,
            calling_cmd: Some("git diff"),
            // delta saw b/a and changed it to a
            expected_displayed_path: "a",
        });
        run_test(FilePathsTestCase {
            name: "b/a from b",
            file_path_relative_to_repo_root: file_path_relative_to_repo_root.as_path(),
            cwd_relative_to_repo_root,
            delta_relative_paths: true,
            input_type,
            calling_cmd: Some("git diff --relative"),
            // delta saw a and didn't change it
            expected_displayed_path: "a",
        });
    }

    #[test]
    fn test_paths_and_hyperlinks_user_in_subdir_file_in_different_subdir() {
        let input_type = InputType::GitDiff;
        let file_path_relative_to_repo_root = PathBuf::from_iter(&["b", "a"]);
        let cwd_relative_to_repo_root = "c";

        run_test(FilePathsTestCase {
            name: "b/a from b",
            file_path_relative_to_repo_root: file_path_relative_to_repo_root.as_path(),
            cwd_relative_to_repo_root,
            delta_relative_paths: false,
            input_type,
            calling_cmd: Some("git diff"),
            expected_displayed_path: "b/a",
        });
    }

    const GIT_DIFF_OUTPUT: &str = r#"
diff --git a/__path__ b/__path__
index 587be6b..975fbec 100644
--- a/__path__
+++ b/__path__
@@ -1 +1 @@
-x
+y
    "#;

    const GIT_GREP_OUTPUT: &str = r#"
__path__:  some matching line
        "#;

    struct FilePathsTestCase<'a> {
        // True location of file in repo
        file_path_relative_to_repo_root: &'a Path,

        // Git spawns delta from repo root so this is only <=> delta's cwd if user invoked git in
        // repo root
        cwd_relative_to_repo_root: &'a str,

        delta_relative_paths: bool,
        input_type: InputType,
        calling_cmd: Option<&'a str>,
        expected_displayed_path: &'a str,
        #[allow(dead_code)]
        name: &'a str,
    }

    #[derive(Debug)]
    enum CallingProcess {
        GitDiff(bool),
        GitGrep,
    }

    #[derive(Clone, Copy, Debug)]
    enum InputType {
        GitDiff,
        GitGrep,
    }

    impl<'a> FilePathsTestCase<'a> {
        pub fn get_args(&self) -> Vec<String> {
            let mut args = vec![
                "--navigate".to_string(), // helps locate the file path in the output
                "--hyperlinks".to_string(),
                "--hyperlinks-file-link-format".to_string(),
                "{path}".to_string(),
            ];
            if self.delta_relative_paths {
                args.push("--relative-paths".to_string());
            }
            args
        }

        pub fn calling_process(&self) -> CallingProcess {
            match (&self.input_type, self.calling_cmd) {
                (InputType::GitDiff, Some(s)) if s.starts_with("git diff --relative") => {
                    CallingProcess::GitDiff(true)
                }
                (InputType::GitDiff, Some(s)) if s.starts_with("git diff") => {
                    CallingProcess::GitDiff(false)
                }
                (InputType::GitGrep, Some(s)) if s.starts_with("git grep") => {
                    CallingProcess::GitGrep
                }
                _ => panic!(
                    "Unexpected calling spec: {:?} {:?}",
                    self.input_type, self.calling_cmd
                ),
            }
        }

        pub fn path_in_git_output(&self) -> String {
            match self.calling_process() {
                CallingProcess::GitDiff(false) => self
                    .file_path_relative_to_repo_root
                    .to_string_lossy()
                    .to_string(),
                CallingProcess::GitDiff(true) => {
                    assert!(self
                        .file_path_relative_to_repo_root
                        .starts_with(self.cwd_relative_to_repo_root));
                    pathdiff::diff_paths(
                        self.file_path_relative_to_repo_root,
                        self.cwd_relative_to_repo_root,
                    )
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
                }
                _ => panic!("Unexpected calling process: {:?}", self.calling_process()),
            }
        }

        pub fn path_in_grep_output(&self) -> String {
            self.file_path_relative_to_repo_root
                .to_string_lossy()
                .to_string()
        }

        pub fn expected_hyperlink_path(&self) -> PathBuf {
            utils::path::fake_delta_cwd_for_tests().join(self.file_path_relative_to_repo_root)
        }
    }

    fn run_test(test_case: FilePathsTestCase) {
        let mut config = integration_test_utils::make_config_from_args(
            &test_case
                .get_args()
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .as_slice(),
        );
        // The test is simulating delta invoked by git hence these are the same
        config.cwd_relative_to_repo_root = Some(test_case.cwd_relative_to_repo_root.to_string());
        config.cwd_of_user_shell_process = utils::path::cwd_of_user_shell_process(
            config.cwd_of_delta_process.as_ref(),
            config.cwd_relative_to_repo_root.as_deref(),
        );
        let mut delta_test = DeltaTest::with_config(&config);
        if let Some(cmd) = test_case.calling_cmd {
            delta_test = delta_test.with_calling_process(cmd)
        }
        let delta_test = match test_case.calling_process() {
            CallingProcess::GitDiff(_) => delta_test
                .with_input(&GIT_DIFF_OUTPUT.replace("__path__", &test_case.path_in_git_output())),
            CallingProcess::GitGrep => delta_test
                .with_input(&GIT_GREP_OUTPUT.replace("__path__", &test_case.path_in_grep_output())),
        };
        delta_test.expect_raw_contains(&format_osc8_hyperlink(
            &PathBuf::from(test_case.expected_hyperlink_path()).to_string_lossy(),
            test_case.expected_displayed_path,
        ));
    }
}
