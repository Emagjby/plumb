use imara_diff::intern::InternedInput;
use imara_diff::{Algorithm, UnifiedDiffBuilder, diff as imara_diff};

pub fn render_baseline_diff(path_label: &str, baseline: &[u8], current: &[u8]) -> String {
    if baseline == current {
        return String::new();
    }

    let Ok(baseline_text) = std::str::from_utf8(baseline) else {
        return format!("Binary files differ: {path_label}\n");
    };
    let Ok(current_text) = std::str::from_utf8(current) else {
        return format!("Binary files differ: {path_label}\n");
    };

    let input = InternedInput::new(baseline_text, current_text);
    let body = imara_diff(
        Algorithm::Histogram,
        &input,
        UnifiedDiffBuilder::new(&input),
    );
    if body.is_empty() {
        return String::new();
    }

    let mut out = String::new();
    out.push_str(&format!("--- baseline: {path_label}\n"));
    out.push_str(&format!("+++ current:  {path_label}\n"));
    out.push_str(&body);
    out
}

#[cfg(test)]
mod tests {
    use super::render_baseline_diff;

    #[test]
    fn render_baseline_diff_empty_when_equal() {
        let baseline = b"hello\nworld\n";
        let current = b"hello\nworld\n";

        let out = render_baseline_diff("src/a.txt", baseline, current);
        assert!(out.is_empty());
    }

    #[test]
    fn render_baseline_diff_one_line_change_contains_headers_and_hunk() {
        let baseline = b"hello\nold\n";
        let current = b"hello\nnew\n";

        let out = render_baseline_diff("src/a.txt", baseline, current);
        assert!(out.contains("--- baseline: src/a.txt"));
        assert!(out.contains("+++ current:  src/a.txt"));
        assert!(out.contains("@@"));
        assert!(out.contains("-old"));
        assert!(out.contains("+new"));
    }

    #[test]
    fn render_baseline_diff_deletion_current_empty_shows_only_removals() {
        let baseline = b"old1\nold2\n";
        let current = b"";

        let out = render_baseline_diff("src/a.txt", baseline, current);
        assert!(out.contains("-old1"));
        assert!(out.contains("-old2"));
        assert!(
            !out.lines()
                .any(|line| line.starts_with('+') && !line.starts_with("+++"))
        );
    }

    #[test]
    fn render_baseline_diff_binary_baseline_or_current_prints_binary_message() {
        let out_binary_baseline = render_baseline_diff("src/a.txt", &[0xff, 0x00], b"text\n");
        let out_binary_current = render_baseline_diff("src/a.txt", b"text\n", &[0xff, 0x00]);

        assert_eq!(out_binary_baseline, "Binary files differ: src/a.txt\n");
        assert_eq!(out_binary_current, "Binary files differ: src/a.txt\n");
    }
}
