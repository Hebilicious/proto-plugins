use anyhow::anyhow;
use proto_pdk::{AnyResult, ParseVersionFileOutput, UnresolvedVersionSpec};
use regex::Regex;
use std::sync::OnceLock;

fn strip_comments(content: &str) -> String {
    content
        .lines()
        .map(|line| line.split(';').next().unwrap_or("").trim_end())
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn normalize_ocaml_version(value: &str) -> String {
    let value = value.trim();
    let value = value.strip_prefix("ocaml-base-compiler.").unwrap_or(value);

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

pub fn parse_ocaml_version(contents: &str) -> AnyResult<Option<UnresolvedVersionSpec>> {
    let value = contents
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("");

    if value.is_empty() {
        return Ok(None);
    }

    let value = normalize_ocaml_version(value);

    if value.contains('.')
        && value
            .chars()
            .next()
            .is_some_and(|char| char.is_ascii_alphabetic())
        && !matches!(value.as_str(), "stable" | "latest" | "canary")
    {
        return Err(anyhow!(
            "Unsupported .ocaml-version value {value}. Use a plain version or ocaml-base-compiler.<version>."
        ));
    }

    Ok(Some(UnresolvedVersionSpec::parse(&value)?))
}

pub fn parse_dune_project_version(contents: &str) -> AnyResult<Option<UnresolvedVersionSpec>> {
    static TOOL_PATTERN: OnceLock<Regex> = OnceLock::new();
    static CONSTRAINT_PATTERN: OnceLock<Regex> = OnceLock::new();
    static EXACT_PATTERN: OnceLock<Regex> = OnceLock::new();

    let tool_pattern = TOOL_PATTERN.get_or_init(|| {
        Regex::new(r#"(?s)\((ocaml-base-compiler|ocaml)((?:\s+\([^)]+\))+)\s*\)"#).unwrap()
    });
    let constraint_pattern = CONSTRAINT_PATTERN.get_or_init(|| {
        Regex::new(r#"\(\s*(>=|<=|=|>|<|\^|~)\s*"?([0-9A-Za-z.+_-]+)"?\s*\)"#).unwrap()
    });
    let exact_pattern = EXACT_PATTERN.get_or_init(|| {
        Regex::new(r#"(?s)\((ocaml-base-compiler|ocaml)\s+"?([0-9A-Za-z.+_-]+)"?\s*\)"#).unwrap()
    });

    let content = strip_comments(contents);

    for caps in tool_pattern.captures_iter(&content) {
        let constraints = constraint_pattern
            .captures_iter(&caps[2])
            .map(|capture| format!("{}{}", &capture[1], normalize_ocaml_version(&capture[2])))
            .collect::<Vec<_>>();

        if !constraints.is_empty() {
            return Ok(Some(UnresolvedVersionSpec::parse(&constraints.join(","))?));
        }
    }

    if let Some(caps) = exact_pattern.captures(&content) {
        return Ok(Some(UnresolvedVersionSpec::parse(
            &normalize_ocaml_version(&caps[2]),
        )?));
    }

    Ok(None)
}

pub fn parse_version_file(file: &str, contents: &str) -> AnyResult<ParseVersionFileOutput> {
    let version = match file {
        ".ocaml-version" => parse_ocaml_version(contents)?,
        "dune-project" => parse_dune_project_version(contents)?,
        _ => None,
    };

    Ok(ParseVersionFileOutput { version })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE_UNSUPPORTED_PACKAGE: &str = "ocaml-variants.5.4.1";

    #[test]
    fn normalizes_prefixed_and_zero_padded_versions() {
        assert_eq!(
            normalize_ocaml_version("ocaml-base-compiler.4.08.1"),
            "4.8.1"
        );
        assert_eq!(normalize_ocaml_version("5.04.0"), "5.4.0");
        assert_eq!(normalize_ocaml_version("stable"), "stable");
    }

    #[test]
    fn parses_ocaml_version_file_with_compiler_prefix() -> AnyResult<()> {
        assert_eq!(
            parse_ocaml_version("ocaml-base-compiler.4.08.1\n")?,
            Some(UnresolvedVersionSpec::parse("4.8.1")?),
        );

        Ok(())
    }

    #[test]
    fn rejects_unsupported_ocaml_version_package_names() {
        let error =
            parse_ocaml_version(FIXTURE_UNSUPPORTED_PACKAGE).expect_err("expected an error");

        assert!(error.to_string().contains(&format!(
            "Unsupported .ocaml-version value {FIXTURE_UNSUPPORTED_PACKAGE}"
        )),);
    }

    #[test]
    fn parses_dune_project_constraints_and_strips_comments() -> AnyResult<()> {
        let contents = r#"
        (lang dune 3.18)

        (package
         (name sample)
         (depends
          ; keep OCaml 4.x for now
          (ocaml (>= 4.08.1) (< 5.0))))
        "#;

        assert_eq!(
            parse_dune_project_version(contents)?,
            Some(UnresolvedVersionSpec::parse(">=4.8.1,<5.0")?),
        );

        Ok(())
    }

    #[test]
    fn parses_exact_dune_project_compiler_pins() -> AnyResult<()> {
        let contents = r#"
        (lang dune 3.18)
        (package
         (name sample)
         (depends
          (ocaml-base-compiler "5.04.0")))
        "#;

        assert_eq!(
            parse_dune_project_version(contents)?,
            Some(UnresolvedVersionSpec::parse("5.4.0")?),
        );

        Ok(())
    }
}
