use crate::version_files::normalize_nushell_version;
#[cfg(target_arch = "wasm32")]
use crate::version_files::parse_version_file as parse_ecosystem_version_file;
#[cfg(target_arch = "wasm32")]
use extism_pdk::*;
use proto_pdk::*;
use std::collections::HashSet;
use std::path::PathBuf;

pub static NAME: &str = "Nushell";
pub static REPOSITORY_URL: &str = "https://github.com/nushell/nushell";

const PLUGIN_EXES: &[&str] = &[
    "nu_plugin_custom_values",
    "nu_plugin_example",
    "nu_plugin_formats",
    "nu_plugin_gstat",
    "nu_plugin_inc",
    "nu_plugin_polars",
    "nu_plugin_query",
    "nu_plugin_stress_internals",
];

pub fn ensure_supported_target(env: &HostEnvironment) -> AnyResult<()> {
    check_supported_os_and_arch(
        NAME,
        env,
        permutations![
            HostOS::Linux => [HostArch::X64, HostArch::Arm64],
            HostOS::MacOS => [HostArch::X64, HostArch::Arm64],
            HostOS::Windows => [HostArch::X64, HostArch::Arm64],
        ],
    )
}

pub fn normalize_nushell_tag(tag: &str) -> Option<String> {
    let tag = normalize_nushell_version(tag);

    if !tag
        .split('.')
        .all(|part| !part.is_empty() && part.chars().all(|char| char.is_ascii_digit()))
    {
        return None;
    }

    (tag.matches('.').count() == 2).then_some(tag)
}

pub fn nushell_target_triple(env: &HostEnvironment) -> AnyResult<String> {
    ensure_supported_target(env)?;

    Ok(get_target_triple(env, NAME)?)
}

pub fn nushell_archive_extension(env: &HostEnvironment) -> &'static str {
    if env.os.is_windows() { "zip" } else { "tar.gz" }
}

pub fn nushell_archive_prefix(version: &VersionSpec, env: &HostEnvironment) -> AnyResult<String> {
    Ok(format!("nu-{version}-{}", nushell_target_triple(env)?))
}

pub fn nushell_asset_name(version: &VersionSpec, env: &HostEnvironment) -> AnyResult<String> {
    Ok(format!(
        "{}.{}",
        nushell_archive_prefix(version, env)?,
        nushell_archive_extension(env),
    ))
}

pub fn nushell_download_url(version: &VersionSpec, env: &HostEnvironment) -> AnyResult<String> {
    let asset = nushell_asset_name(version, env)?;

    Ok(format!(
        "{REPOSITORY_URL}/releases/download/{version}/{asset}"
    ))
}

pub fn nushell_checksum_url(version: &VersionSpec) -> String {
    format!("{REPOSITORY_URL}/releases/download/{version}/SHA256SUMS")
}

pub fn build_register_tool_output() -> AnyResult<RegisterToolOutput> {
    Ok(RegisterToolOutput {
        name: NAME.into(),
        type_of: PluginType::CommandLine,
        default_version: Some(UnresolvedVersionSpec::Alias("stable".into())),
        minimum_proto_version: Some(Version::new(0, 57, 0)),
        plugin_version: Version::parse(env!("CARGO_PKG_VERSION")).ok(),
        ..RegisterToolOutput::default()
    })
}

pub fn build_load_versions_output(tags: Vec<String>) -> AnyResult<LoadVersionsOutput> {
    let mut seen = HashSet::new();
    let versions = tags
        .into_iter()
        .filter_map(|tag| normalize_nushell_tag(&tag))
        .filter(|tag| seen.insert(tag.clone()))
        .collect::<Vec<_>>();
    let mut output = LoadVersionsOutput::from(versions)?;

    if let Some(latest) = output.latest.clone() {
        output.aliases.insert("stable".into(), latest);
    }

    Ok(output)
}

pub fn build_resolve_version_output(initial: &UnresolvedVersionSpec) -> ResolveVersionOutput {
    let mut output = ResolveVersionOutput::default();

    if let UnresolvedVersionSpec::Alias(alias) = initial
        && alias == "stable"
    {
        output.candidate = Some(UnresolvedVersionSpec::Alias("latest".into()));
    }

    output
}

pub fn build_detect_version_output() -> DetectVersionOutput {
    DetectVersionOutput {
        files: vec![".nu-version".into(), ".nushell-version".into()],
        ignore: vec![],
    }
}

pub fn build_download_prebuilt_output(
    version: &VersionSpec,
    env: &HostEnvironment,
) -> AnyResult<DownloadPrebuiltOutput> {
    let archive_prefix = nushell_archive_prefix(version, env)?;
    let asset = nushell_asset_name(version, env)?;

    Ok(DownloadPrebuiltOutput {
        archive_prefix: Some(archive_prefix),
        checksum: None,
        checksum_name: Some("SHA256SUMS".into()),
        checksum_public_key: None,
        checksum_url: Some(nushell_checksum_url(version)),
        download_name: Some(asset),
        download_url: nushell_download_url(version, env)?,
        http_headers: Default::default(),
        post_script: None,
    })
}

pub fn build_locate_executables_output(env: &HostEnvironment) -> LocateExecutablesOutput {
    let mut exes = LocateExecutablesOutput::default().exes;

    exes.insert(
        "nu".into(),
        ExecutableConfig::new_primary(env.os.get_exe_name("nu")),
    );

    for exe in PLUGIN_EXES {
        exes.insert(
            (*exe).into(),
            ExecutableConfig::new(env.os.get_exe_name(exe)),
        );
    }

    LocateExecutablesOutput {
        exes,
        exes_dirs: vec![PathBuf::from(".")],
        ..LocateExecutablesOutput::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proto_pdk::HostLibc;

    const FIXTURE_LATEST_VERSION: &str = "0.112.2";
    const FIXTURE_PREVIOUS_VERSION: &str = "0.111.0";

    fn host_env(os: HostOS, arch: HostArch, libc: HostLibc) -> HostEnvironment {
        HostEnvironment {
            arch,
            ci: false,
            libc,
            os,
            home_dir: VirtualPath::Real(PathBuf::from("/home/tester")),
        }
    }

    #[test]
    fn normalizes_supported_nushell_tags_only() {
        assert_eq!(
            normalize_nushell_tag(FIXTURE_LATEST_VERSION),
            Some(FIXTURE_LATEST_VERSION.into()),
        );
        assert_eq!(
            normalize_nushell_tag(&format!("v{FIXTURE_LATEST_VERSION}")),
            Some(FIXTURE_LATEST_VERSION.into()),
        );
        assert_eq!(normalize_nushell_tag("0.112"), None);
        assert_eq!(normalize_nushell_tag("nightly"), None);
        assert_eq!(normalize_nushell_tag("0.112.2+meta"), None);
    }

    #[test]
    fn builds_versions_output_with_stable_alias_and_deduped_versions() -> AnyResult<()> {
        let output = build_load_versions_output(vec![
            "v0.111.0".into(),
            FIXTURE_PREVIOUS_VERSION.into(),
            FIXTURE_LATEST_VERSION.into(),
            "nightly".into(),
        ])?;

        assert_eq!(
            output.latest,
            Some(UnresolvedVersionSpec::parse(FIXTURE_LATEST_VERSION)?),
        );
        assert_eq!(
            output.aliases.get("stable"),
            Some(&UnresolvedVersionSpec::parse(FIXTURE_LATEST_VERSION)?),
        );
        assert_eq!(
            output.versions,
            vec![
                VersionSpec::parse(FIXTURE_PREVIOUS_VERSION)?,
                VersionSpec::parse(FIXTURE_LATEST_VERSION)?,
            ],
        );

        Ok(())
    }

    #[test]
    fn rewrites_stable_alias_to_latest() {
        let output = build_resolve_version_output(&UnresolvedVersionSpec::Alias("stable".into()));

        assert_eq!(
            output.candidate,
            Some(UnresolvedVersionSpec::Alias("latest".into())),
        );
    }

    #[test]
    fn declares_ecosystem_file_precedence() {
        let output = build_detect_version_output();

        assert_eq!(output.files, vec![".nu-version", ".nushell-version"]);
        assert!(output.ignore.is_empty());
    }

    #[test]
    fn resolves_release_assets_for_supported_targets() -> AnyResult<()> {
        let version = VersionSpec::parse(FIXTURE_LATEST_VERSION)?;
        let cases = [
            (
                host_env(HostOS::Linux, HostArch::X64, HostLibc::Gnu),
                "nu-0.112.2-x86_64-unknown-linux-gnu.tar.gz",
            ),
            (
                host_env(HostOS::Linux, HostArch::Arm64, HostLibc::Musl),
                "nu-0.112.2-aarch64-unknown-linux-musl.tar.gz",
            ),
            (
                host_env(HostOS::MacOS, HostArch::Arm64, HostLibc::Gnu),
                "nu-0.112.2-aarch64-apple-darwin.tar.gz",
            ),
            (
                host_env(HostOS::Windows, HostArch::Arm64, HostLibc::Gnu),
                "nu-0.112.2-aarch64-pc-windows-msvc.zip",
            ),
        ];

        for (env, expected) in cases {
            assert_eq!(nushell_asset_name(&version, &env)?, expected);
        }

        Ok(())
    }

    #[test]
    fn builds_prebuilt_download_output_with_checksum_file() -> AnyResult<()> {
        let version = VersionSpec::parse(FIXTURE_LATEST_VERSION)?;
        let env = host_env(HostOS::MacOS, HostArch::Arm64, HostLibc::Gnu);
        let output = build_download_prebuilt_output(&version, &env)?;

        assert_eq!(
            output.archive_prefix,
            Some("nu-0.112.2-aarch64-apple-darwin".into()),
        );
        assert_eq!(
            output.download_name,
            Some("nu-0.112.2-aarch64-apple-darwin.tar.gz".into()),
        );
        assert_eq!(output.checksum_name, Some("SHA256SUMS".into()),);
        assert_eq!(
            output.download_url,
            "https://github.com/nushell/nushell/releases/download/0.112.2/nu-0.112.2-aarch64-apple-darwin.tar.gz",
        );
        assert_eq!(
            output.checksum_url,
            Some("https://github.com/nushell/nushell/releases/download/0.112.2/SHA256SUMS".into()),
        );

        Ok(())
    }

    #[test]
    fn exposes_nu_as_primary_executable_and_plugins_as_secondary() {
        let output = build_locate_executables_output(&host_env(
            HostOS::Linux,
            HostArch::Arm64,
            HostLibc::Gnu,
        ));

        assert_eq!(output.exes_dirs, vec![PathBuf::from(".")]);
        assert_eq!(output.exes["nu"], ExecutableConfig::new_primary("nu"));
        assert_eq!(
            output.exes["nu_plugin_query"],
            ExecutableConfig::new("nu_plugin_query"),
        );
    }
}

#[cfg(target_arch = "wasm32")]
#[plugin_fn]
pub fn register_tool(Json(_): Json<RegisterToolInput>) -> FnResult<Json<RegisterToolOutput>> {
    Ok(Json(build_register_tool_output()?))
}

#[cfg(target_arch = "wasm32")]
#[plugin_fn]
pub fn load_versions(Json(_): Json<LoadVersionsInput>) -> FnResult<Json<LoadVersionsOutput>> {
    Ok(Json(build_load_versions_output(load_git_tags(
        REPOSITORY_URL,
    )?)?))
}

#[cfg(target_arch = "wasm32")]
#[plugin_fn]
pub fn resolve_version(
    Json(input): Json<ResolveVersionInput>,
) -> FnResult<Json<ResolveVersionOutput>> {
    Ok(Json(build_resolve_version_output(&input.initial)))
}

#[cfg(target_arch = "wasm32")]
#[plugin_fn]
pub fn detect_version_files(_: ()) -> FnResult<Json<DetectVersionOutput>> {
    Ok(Json(build_detect_version_output()))
}

#[cfg(target_arch = "wasm32")]
#[plugin_fn]
pub fn parse_version_file(
    Json(input): Json<ParseVersionFileInput>,
) -> FnResult<Json<ParseVersionFileOutput>> {
    Ok(Json(parse_ecosystem_version_file(
        &input.file,
        &input.content,
    )?))
}

#[cfg(target_arch = "wasm32")]
#[plugin_fn]
pub fn download_prebuilt(
    Json(input): Json<DownloadPrebuiltInput>,
) -> FnResult<Json<DownloadPrebuiltOutput>> {
    let env = get_host_environment()?;

    Ok(Json(build_download_prebuilt_output(
        &input.context.version,
        &env,
    )?))
}

#[cfg(target_arch = "wasm32")]
#[plugin_fn]
pub fn locate_executables(
    Json(_input): Json<LocateExecutablesInput>,
) -> FnResult<Json<LocateExecutablesOutput>> {
    Ok(Json(build_locate_executables_output(
        &get_host_environment()?,
    )))
}
