/// Whether `key` already has the shape of a name key: non-empty, single-spaced, trimmed and
/// lowercase. The interface computes keys (adding Unicode NFC, which Rust's standard library
/// cannot check); this rejects keys that clearly were not normalized, so a caller bug cannot
/// defeat the unique-name indexes. Whitespace follows the Unicode `White_Space` property, the
/// same set the TypeScript normalization uses.
pub fn is_normalized_key(key: &str) -> bool {
    !key.is_empty()
        && key == key.split_whitespace().collect::<Vec<_>>().join(" ")
        && key == key.to_lowercase()
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::is_normalized_key;

    #[derive(Deserialize)]
    struct Case {
        input: String,
        key: String,
    }

    #[derive(Deserialize)]
    struct Fixture {
        cases: Vec<Case>,
    }

    #[test]
    fn every_key_the_interface_produces_is_accepted() {
        let fixture: Fixture =
            serde_json::from_str(include_str!("../../contract/name-keys.json")).unwrap();
        assert!(!fixture.cases.is_empty());
        for case in fixture.cases {
            assert!(
                is_normalized_key(&case.key),
                "{:?} -> {:?}",
                case.input,
                case.key
            );
        }
    }

    #[test]
    fn keys_that_were_not_normalized_are_rejected() {
        for key in [
            "",
            " ",
            "Conta",
            "conta  corrente",
            " conta",
            "conta ",
            "AÇÃO",
            "a\u{a0}\u{a0}b",
        ] {
            assert!(!is_normalized_key(key), "{key:?}");
        }
        assert!(is_normalized_key("conta corrente"));
    }
}
