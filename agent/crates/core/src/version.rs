use semver::Version;

/// Returns `true` if candidate version string is strictly greater/newer than current version string.
/// Handles leading 'v' prefix gracefully (e.g. "v1.0.2" vs "1.0.1").
pub fn is_newer_version(candidate: &str, current: &str) -> bool {
    let clean_c = candidate.trim().trim_start_matches('v');
    let clean_curr = current.trim().trim_start_matches('v');

    match (Version::parse(clean_c), Version::parse(clean_curr)) {
        (Ok(c), Ok(curr)) => c > curr,
        _ => {
            // Fallback comparison for non-strict semver: compare numeric segments
            let parse_parts = |s: &str| -> Vec<u64> {
                s.split(['.', '-', '+'])
                    .filter_map(|p| p.parse::<u64>().ok())
                    .collect()
            };
            let c_parts = parse_parts(clean_c);
            let curr_parts = parse_parts(clean_curr);
            c_parts > curr_parts
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version() {
        // Obsolete version should NOT be treated as update
        assert!(!is_newer_version("0.1.1", "1.0.2"));
        assert!(!is_newer_version("v0.1.1", "1.0.2"));
        assert!(!is_newer_version("1.0.2", "1.0.2"));
        assert!(!is_newer_version("v1.0.2", "v1.0.2"));
        assert!(!is_newer_version("1.0.1", "1.0.2"));

        // Strictly newer version SHOULD be treated as update
        assert!(is_newer_version("1.0.3", "1.0.2"));
        assert!(is_newer_version("v1.1.0", "1.0.2"));
        assert!(is_newer_version("2.0.0", "1.0.2"));
        assert!(is_newer_version("1.0.2-beta.2", "1.0.2-beta.1"));
    }
}
