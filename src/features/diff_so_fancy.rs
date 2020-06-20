use crate::features::diff_highlight;
use crate::features::FeatureValueFunction;

pub fn make_feature() -> Vec<(String, FeatureValueFunction)> {
    let mut feature = diff_highlight::_make_feature(true);
    feature.extend(builtin_feature!([
        (
            "minus-emph-style",
            String,
            Some("color.diff-highlight.oldHighlight"),
            _opt => "bold red 52"
        ),
        (
            "plus-emph-style",
            String,
            Some("color.diff-highlight.newHighlight"),
            _opt => "bold green 22"
        ),
        (
            "commit-style",
            String,
            None,
            _opt => "bold yellow"
        ),
        (
            "commit-decoration-style",
            String,
            None,
            _opt => "none"
        ),
        (
            "file-style",
            String,
            Some("color.diff.meta"),
            _opt => "11"
        ),
        (
            "file-decoration-style",
            String,
            None,
            _opt => "bold yellow ul ol"
        ),
        (
            "hunk-header-style",
            String,
            Some("color.diff.frag"),
            _opt => "bold syntax"
        ),
        (
            "hunk-header-decoration-style",
            String,
            None,
            _opt => "magenta box"
        )
    ]));
    feature
}