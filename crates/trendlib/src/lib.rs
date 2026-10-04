#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]

/// The version of this crate, shared across the whole workspace.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn version_is_three_numeric_components() {
        let core: &str = VERSION.split(['-', '+']).next().unwrap();
        let parts: Vec<&str> = core.split('.').collect();
        assert_eq!(parts.len(), 3, "version {VERSION} is not major.minor.patch");
        assert!(parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())));
    }
}
