use anyhow::anyhow;
use proto_pdk::{AnyResult, ParseVersionFileOutput, UnresolvedVersionSpec};
use regex::Regex;
use std::sync::OnceLock;

pub fn normalize_nushell_version(value: &str) -> String {
    let value = value.trim();
    let value = value.strip_prefix('v').unwrap_or(value);
    let value = value.strip_prefix("nu-").unwrap_or(value);

    if value
        .split('.')
        .all(|part| !part.is_empty() && part.chars().all(|char| char.is_ascii_digit()))
    {
        return value
            .split('.')
            .map(|part| part.parse::<u64>().unwrap().to_string())
            .collect::<Vec<_>>()
            .join(".");
    }

    value.to_owned()
}

pub fn parse_nushell_version(contents: &str) -> AnyResult<Option<UnresolvedVersionSpec>> {
    static UNSUPPORTED_PACKAGE_PATTERN: OnceLock<Regex> = OnceLock::new();
    let pattern = UNSUPPORTED_PACKAGE_PATTERN
        .get_or_init(|| Regex::new(r#"^[A-Za-z][A-Za-z0-9_-]+\.[0-9]"#).unwrap());

    let value = contents
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("");

    if value.is_empty() {
        return Ok(None);
    }

    let value = normalize_nushell_version(value);

    if pattern.is_match(&value) && !matches!(value.as_str(), "stable" | "latest" | "canary") {
        return Err(anyhow!(
            "Unsupported Nushell version value {value}. Use a plain version, nu-<version>, or an alias."
        ));
    }

    Ok(Some(UnresolvedVersionSpec::parse(&value)?))
}

pub fn parse_version_file(file: &str, contents: &str) -> AnyResult<ParseVersionFileOutput> {
    let version = match file {
        ".nu-version" | ".nushell-version" => parse_nushell_version(contents)?,
        _ => None,
    };

    Ok(ParseVersionFileOutput { version })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_prefixed_and_zero_padded_versions() {
        assert_eq!(normalize_nushell_version("v0.112.2"), "0.112.2");
        assert_eq!(normalize_nushell_version("nu-0.112.2"), "0.112.2");
        assert_eq!(normalize_nushell_version("0.0112.002"), "0.112.2");
        assert_eq!(normalize_nushell_version("stable"), "stable");
    }

    #[test]
    fn parses_nushell_version_file_with_prefix() -> AnyResult<()> {
        assert_eq!(
            parse_nushell_version("nu-0.112.2\n")?,
            Some(UnresolvedVersionSpec::parse("0.112.2")?),
        );

        Ok(())
    }

    #[test]
    fn parses_nushell_aliases() -> AnyResult<()> {
        assert_eq!(
            parse_nushell_version("stable\n")?,
            Some(UnresolvedVersionSpec::parse("stable")?),
        );

        Ok(())
    }

    #[test]
    fn rejects_unsupported_package_like_values() {
        let error = parse_nushell_version("nushell.0.112.2").expect_err("expected an error");

        assert!(
            error
                .to_string()
                .contains("Unsupported Nushell version value nushell.0.112.2"),
        );
    }
}
