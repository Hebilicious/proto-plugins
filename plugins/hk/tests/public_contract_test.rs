use hk::{build_download_prebuilt_output, build_locate_executables_output, parse_version_file};
use proto_pdk::{
    AnyResult, ExecutableConfig, HostArch, HostEnvironment, HostLibc, HostOS,
    UnresolvedVersionSpec, VersionSpec, VirtualPath,
};
use std::path::PathBuf;

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
fn public_parse_version_file_normalizes_supported_hk_inputs() -> AnyResult<()> {
    let output = parse_version_file(".hk-version", "v1.46.0\n")?;

    assert_eq!(
        output.version,
        Some(UnresolvedVersionSpec::parse("1.46.0")?)
    );

    Ok(())
}

#[test]
fn public_download_prebuilt_output_uses_linux_release_archive() -> AnyResult<()> {
    let output = build_download_prebuilt_output(
        &VersionSpec::parse("1.46.0")?,
        &host_env(HostOS::Linux, HostArch::X64, HostLibc::Gnu),
    )?;

    assert_eq!(
        output.download_url,
        "https://github.com/jdx/hk/releases/download/v1.46.0/hk-x86_64-unknown-linux-gnu.tar.gz",
    );
    assert_eq!(
        output.download_name,
        Some("hk-x86_64-unknown-linux-gnu.tar.gz".into()),
    );

    Ok(())
}

#[test]
fn public_locate_executables_output_exposes_hk() {
    let output =
        build_locate_executables_output(&host_env(HostOS::Linux, HostArch::X64, HostLibc::Gnu));

    assert_eq!(output.exes_dirs, vec![PathBuf::from(".")]);
    assert_eq!(output.exes["hk"], ExecutableConfig::new_primary("hk"));
}
