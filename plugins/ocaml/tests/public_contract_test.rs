use ocaml::{build_activate_environment_output, parse_version_file};
use proto_pdk::{
    AnyResult, HostArch, HostEnvironment, HostLibc, HostOS, UnresolvedVersionSpec, VirtualPath,
};
use std::path::PathBuf;

const FIXTURE_OCAML_VERSION: &str = "5.4.1";

fn host_env() -> HostEnvironment {
    HostEnvironment {
        arch: HostArch::Arm64,
        ci: false,
        libc: HostLibc::Gnu,
        os: HostOS::Linux,
        home_dir: VirtualPath::Real(PathBuf::from("/home/tester")),
    }
}

fn tool_dir() -> VirtualPath {
    VirtualPath::Virtual {
        path: PathBuf::from(format!("/proto/tools/ocaml/{FIXTURE_OCAML_VERSION}")),
        virtual_prefix: PathBuf::from("/proto"),
        real_prefix: PathBuf::from("/root/.proto"),
    }
}

fn real_tool_dir() -> String {
    format!("/root/.proto/tools/ocaml/{FIXTURE_OCAML_VERSION}")
}

fn real_tool_path(path: &str) -> String {
    format!("{}/{path}", real_tool_dir())
}

#[test]
fn public_parse_version_file_normalizes_supported_ocaml_inputs() -> AnyResult<()> {
    assert_eq!(
        parse_version_file(".ocaml-version", "ocaml-base-compiler.4.08.1")?.version,
        Some(UnresolvedVersionSpec::parse("4.8.1")?),
    );

    assert_eq!(
        parse_version_file(
            "dune-project",
            r#"
            (lang dune 3.18)
            (package
             (name sample)
             (depends (ocaml (>= 5.04.0) (< 5.5))))
            "#,
        )?
        .version,
        Some(UnresolvedVersionSpec::parse(">=5.4.0,<5.5")?),
    );

    Ok(())
}

#[test]
fn public_activate_environment_output_keeps_tool_paths_from_opam_env() {
    let expected_opam_root = real_tool_path(".opam-root");
    let expected_switch_prefix = real_tool_path("_opam");

    let output = build_activate_environment_output(
        &format!(
            r#"(("OPAMROOT" "{0}/.opam-root")
            ("OPAMSWITCH" "{0}")
            ("OPAM_SWITCH_PREFIX" "{0}/_opam")
            ("PATH" "{0}/bin:{0}/_opam/bin:/root/.cargo/bin:/usr/bin"))"#,
            real_tool_dir(),
        ),
        &tool_dir(),
        &host_env(),
    );

    assert_eq!(
        output.paths,
        vec![
            PathBuf::from(real_tool_path("bin")),
            PathBuf::from(real_tool_path("_opam/bin")),
        ],
    );
    assert_eq!(output.env.get("OPAMROOT"), Some(&expected_opam_root),);
    assert_eq!(
        output.env.get("OPAM_SWITCH_PREFIX"),
        Some(&expected_switch_prefix),
    );
}
