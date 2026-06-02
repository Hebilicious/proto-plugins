use nu::{build_download_prebuilt_output, build_locate_executables_output, parse_version_file};
use proto_pdk::{
    AnyResult, ExecutableConfig, HostArch, HostEnvironment, HostLibc, HostOS,
    UnresolvedVersionSpec, VersionSpec, VirtualPath,
};
use std::path::PathBuf;

const FIXTURE_NU_VERSION: &str = "0.112.2";

fn host_env() -> HostEnvironment {
    HostEnvironment {
        arch: HostArch::Arm64,
        ci: false,
        libc: HostLibc::Gnu,
        os: HostOS::Linux,
        home_dir: VirtualPath::Real(PathBuf::from("/home/tester")),
    }
}

#[test]
fn public_parse_version_file_normalizes_supported_nushell_inputs() -> AnyResult<()> {
    assert_eq!(
        parse_version_file(".nu-version", "nu-0.112.2")?.version,
        Some(UnresolvedVersionSpec::parse(FIXTURE_NU_VERSION)?),
    );

    assert_eq!(
        parse_version_file(".nushell-version", "v0.112.2")?.version,
        Some(UnresolvedVersionSpec::parse(FIXTURE_NU_VERSION)?),
    );

    Ok(())
}

#[test]
fn public_download_prebuilt_output_uses_linux_release_archive() -> AnyResult<()> {
    let output =
        build_download_prebuilt_output(&VersionSpec::parse(FIXTURE_NU_VERSION)?, &host_env())?;

    assert_eq!(
        output.archive_prefix,
        Some("nu-0.112.2-aarch64-unknown-linux-gnu".into()),
    );
    assert_eq!(
        output.download_url,
        "https://github.com/nushell/nushell/releases/download/0.112.2/nu-0.112.2-aarch64-unknown-linux-gnu.tar.gz",
    );
    assert_eq!(
        output.checksum_url,
        Some("https://github.com/nushell/nushell/releases/download/0.112.2/SHA256SUMS".into()),
    );

    Ok(())
}

#[test]
fn public_locate_executables_output_exposes_nu_and_bundled_plugins() {
    let output = build_locate_executables_output(&host_env());

    assert_eq!(output.exes["nu"], ExecutableConfig::new_primary("nu"));
    assert_eq!(
        output.exes["nu_plugin_formats"],
        ExecutableConfig::new("nu_plugin_formats"),
    );
}
