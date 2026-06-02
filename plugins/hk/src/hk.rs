use crate::version_files::normalize_hk_version;
#[cfg(target_arch = "wasm32")]
use crate::version_files::parse_version_file as parse_ecosystem_version_file;
#[cfg(target_arch = "wasm32")]
use extism_pdk::*;
use proto_pdk::*;
use std::collections::HashSet;
use std::path::PathBuf;

pub static NAME: &str = "HK";
pub static REPOSITORY_URL: &str = "https://github.com/jdx/hk";

pub fn ensure_supported_target(env: &HostEnvironment) -> AnyResult<()> {
    check_supported_os_and_arch(
        NAME,
        env,
        permutations![
            HostOS::Linux => [HostArch::X64, HostArch::Arm64],
            HostOS::MacOS => [HostArch::Arm64],
            HostOS::Windows => [HostArch::X64, HostArch::Arm64],
        ],
    )
}

pub fn normalize_hk_tag(tag: &str) -> Option<String> {
    let tag = normalize_hk_version(tag);

    if !tag
        .split('.')
        .all(|part| !part.is_empty() && part.chars().all(|char| char.is_ascii_digit()))
    {
        return None;
    }

    (tag.matches('.').count() == 2).then_some(tag)
}

pub fn hk_target_triple(env: &HostEnvironment) -> AnyResult<String> {
    ensure_supported_target(env)?;

    Ok(get_target_triple(env, NAME)?)
}

pub fn hk_archive_extension(env: &HostEnvironment) -> &'static str {
    if env.os.is_windows() { "zip" } else { "tar.gz" }
}

pub fn hk_archive_prefix(env: &HostEnvironment) -> AnyResult<String> {
    Ok(format!("hk-{}", hk_target_triple(env)?))
}

pub fn hk_asset_name(env: &HostEnvironment) -> AnyResult<String> {
    Ok(format!(
        "{}.{}",
        hk_archive_prefix(env)?,
        hk_archive_extension(env),
    ))
}

pub fn hk_download_url(version: &VersionSpec, env: &HostEnvironment) -> AnyResult<String> {
    let asset = hk_asset_name(env)?;

    Ok(format!(
        "{REPOSITORY_URL}/releases/download/v{version}/{asset}"
    ))
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
        .filter_map(|tag| normalize_hk_tag(&tag))
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
        files: vec![".hk-version".into()],
        ignore: vec![],
    }
}

pub fn build_download_prebuilt_output(
    version: &VersionSpec,
    env: &HostEnvironment,
) -> AnyResult<DownloadPrebuiltOutput> {
    let archive_prefix = hk_archive_prefix(env)?;
    let asset = hk_asset_name(env)?;

    Ok(DownloadPrebuiltOutput {
        archive_prefix: Some(archive_prefix),
        checksum: None,
        checksum_name: None,
        checksum_public_key: None,
        checksum_url: None,
        download_name: Some(asset),
        download_url: hk_download_url(version, env)?,
        http_headers: Default::default(),
        post_script: None,
    })
}

pub fn build_locate_executables_output(env: &HostEnvironment) -> LocateExecutablesOutput {
    LocateExecutablesOutput {
        exes: [(
            "hk".into(),
            ExecutableConfig::new_primary(env.os.get_exe_name("hk")),
        )]
        .into_iter()
        .collect(),
        exes_dirs: vec![PathBuf::from(".")],
        ..LocateExecutablesOutput::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proto_pdk::HostLibc;

    const FIXTURE_LATEST_VERSION: &str = "1.46.0";
    const FIXTURE_PREVIOUS_VERSION: &str = "1.45.0";

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
    fn normalizes_supported_hk_tags_only() {
        assert_eq!(
            normalize_hk_tag(FIXTURE_LATEST_VERSION),
            Some(FIXTURE_LATEST_VERSION.into()),
        );
        assert_eq!(
            normalize_hk_tag(&format!("v{FIXTURE_LATEST_VERSION}")),
            Some(FIXTURE_LATEST_VERSION.into()),
        );
        assert_eq!(normalize_hk_tag("1.46"), None);
        assert_eq!(normalize_hk_tag("nightly"), None);
        assert_eq!(normalize_hk_tag("1.46.0+meta"), None);
    }

    #[test]
    fn builds_versions_output_with_stable_alias_and_deduped_versions() -> AnyResult<()> {
        let output = build_load_versions_output(vec![
            "v1.45.0".into(),
            FIXTURE_PREVIOUS_VERSION.into(),
            FIXTURE_LATEST_VERSION.into(),
            "hk@1.46.0".into(),
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
    fn declares_ecosystem_file() {
        let output = build_detect_version_output();

        assert_eq!(output.files, vec![".hk-version"]);
        assert!(output.ignore.is_empty());
    }

    #[test]
    fn resolves_release_assets_for_supported_targets() -> AnyResult<()> {
        let cases = [
            (
                host_env(HostOS::Linux, HostArch::X64, HostLibc::Gnu),
                "hk-x86_64-unknown-linux-gnu.tar.gz",
            ),
            (
                host_env(HostOS::Linux, HostArch::Arm64, HostLibc::Musl),
                "hk-aarch64-unknown-linux-musl.tar.gz",
            ),
            (
                host_env(HostOS::MacOS, HostArch::Arm64, HostLibc::Gnu),
                "hk-aarch64-apple-darwin.tar.gz",
            ),
            (
                host_env(HostOS::Windows, HostArch::Arm64, HostLibc::Gnu),
                "hk-aarch64-pc-windows-msvc.zip",
            ),
        ];

        for (env, expected) in cases {
            assert_eq!(hk_asset_name(&env)?, expected);
        }

        Ok(())
    }

    #[test]
    fn builds_prebuilt_download_output() -> AnyResult<()> {
        let version = VersionSpec::parse(FIXTURE_LATEST_VERSION)?;
        let env = host_env(HostOS::Linux, HostArch::X64, HostLibc::Gnu);
        let output = build_download_prebuilt_output(&version, &env)?;

        assert_eq!(
            output.archive_prefix,
            Some("hk-x86_64-unknown-linux-gnu".into()),
        );
        assert_eq!(
            output.download_name,
            Some("hk-x86_64-unknown-linux-gnu.tar.gz".into()),
        );
        assert_eq!(
            output.download_url,
            "https://github.com/jdx/hk/releases/download/v1.46.0/hk-x86_64-unknown-linux-gnu.tar.gz",
        );

        Ok(())
    }

    #[test]
    fn exposes_hk_as_primary_executable() {
        let output =
            build_locate_executables_output(&host_env(HostOS::Linux, HostArch::X64, HostLibc::Gnu));

        assert_eq!(output.exes_dirs, vec![PathBuf::from(".")]);
        assert_eq!(output.exes["hk"], ExecutableConfig::new_primary("hk"));
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
