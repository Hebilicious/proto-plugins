use proto_pdk::{
    AnyResult, HostArch, HostEnvironment, HostOS, PluginContext, Version, VersionSpec, VirtualPath,
};
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

pub const OPAM_RELEASE_VERSION: &str = "2.5.0";
pub const OPAM_ROOT_DIR: &str = ".opam-root";
pub const OPAM_SWITCH_DIR: &str = "_opam";
pub const OPAM_BIN_DIR: &str = "bin";
pub const OPAM_REPOSITORY_NAME: &str = "default";
pub const OPAM_REPOSITORY_URL: &str = "https://opam.ocaml.org";

#[derive(Clone, Debug, PartialEq)]
pub struct CommandPlan {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<VirtualPath>,
    pub env: HashMap<String, String>,
}

impl CommandPlan {
    pub fn new(command: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            command: command.into(),
            args,
            cwd: None,
            env: HashMap::new(),
        }
    }

    pub fn with_cwd(mut self, cwd: &VirtualPath) -> Self {
        self.cwd = Some(cwd.to_owned());
        self
    }
}

pub fn opam_release_version() -> Version {
    Version::parse(OPAM_RELEASE_VERSION).expect("valid opam release version")
}

pub fn opam_binary_name(env: &HostEnvironment) -> &'static str {
    if env.os.is_windows() {
        "opam.exe"
    } else {
        "opam"
    }
}

pub fn opam_install_bin(env: &HostEnvironment) -> PathBuf {
    PathBuf::from(OPAM_BIN_DIR).join(opam_binary_name(env))
}

pub fn opam_switch_bin_dir() -> PathBuf {
    PathBuf::from(OPAM_SWITCH_DIR).join(OPAM_BIN_DIR)
}

pub fn opam_root_dir(install_dir: &VirtualPath) -> VirtualPath {
    install_dir.join(OPAM_ROOT_DIR)
}

pub fn opam_switch_prefix(install_dir: &VirtualPath) -> VirtualPath {
    install_dir.join(OPAM_SWITCH_DIR)
}

pub fn opam_executable_path(tool_dir: &VirtualPath, env: &HostEnvironment) -> VirtualPath {
    tool_dir.join(opam_install_bin(env))
}

pub fn compiler_package(version: &VersionSpec) -> String {
    format!("ocaml-base-compiler.{version}")
}

pub fn opam_asset_name(env: &HostEnvironment, version: &Version) -> AnyResult<String> {
    let os = match env.os {
        HostOS::Linux => "linux",
        HostOS::MacOS => "macos",
        HostOS::Windows => "windows.exe",
        _ => {
            return Err(proto_pdk::PluginError::UnsupportedOS {
                tool: "OCaml".into(),
                os: env.os.to_string(),
            }
            .into());
        }
    };

    let arch = match env.arch {
        HostArch::X64 => "x86_64",
        HostArch::Arm64 => "arm64",
        _ => {
            return Err(proto_pdk::PluginError::UnsupportedTarget {
                tool: "OCaml".into(),
                arch: env.arch.to_string(),
                os: env.os.to_string(),
            }
            .into());
        }
    };

    Ok(if env.os.is_windows() {
        format!("opam-{version}-{arch}-{os}")
    } else {
        format!("opam-{version}-{arch}-{os}")
    })
}

pub fn opam_download_url(env: &HostEnvironment, version: &Version) -> AnyResult<String> {
    let asset = opam_asset_name(env, version)?;

    Ok(format!(
        "https://github.com/ocaml/opam/releases/download/{version}/{asset}"
    ))
}

pub fn build_opam_init_command(
    opam: &str,
    install_dir: &VirtualPath,
    env: &HostEnvironment,
) -> CommandPlan {
    let mut args = vec![
        "init".into(),
        "--yes".into(),
        "--bare".into(),
        "--no-setup".into(),
        "--disable-sandboxing".into(),
        "--bypass-checks".into(),
        "--no-opamrc".into(),
        "--root".into(),
        opam_root_dir(install_dir)
            .real_path_string()
            .unwrap_or_else(|| opam_root_dir(install_dir).to_string()),
        OPAM_REPOSITORY_NAME.into(),
        OPAM_REPOSITORY_URL.into(),
    ];

    if env.os.is_windows() {
        args.push("--cygwin-internal-install".into());
    }

    CommandPlan::new(opam, args).with_cwd(install_dir)
}

pub fn build_switch_create_command(
    opam: &str,
    install_dir: &VirtualPath,
    version: &VersionSpec,
) -> CommandPlan {
    let switch_dir = install_dir
        .real_path_string()
        .unwrap_or_else(|| install_dir.to_string());

    CommandPlan::new(
        opam,
        vec![
            "switch".into(),
            "create".into(),
            switch_dir.clone(),
            compiler_package(version),
            "--yes".into(),
            "--root".into(),
            opam_root_dir(install_dir)
                .real_path_string()
                .unwrap_or_else(|| opam_root_dir(install_dir).to_string()),
        ],
    )
    .with_cwd(install_dir)
}

pub fn build_dune_install_command(opam: &str, install_dir: &VirtualPath) -> CommandPlan {
    let switch_dir = install_dir
        .real_path_string()
        .unwrap_or_else(|| install_dir.to_string());

    CommandPlan::new(
        opam,
        vec![
            "install".into(),
            "dune".into(),
            "--yes".into(),
            "--root".into(),
            opam_root_dir(install_dir)
                .real_path_string()
                .unwrap_or_else(|| opam_root_dir(install_dir).to_string()),
            "--switch".into(),
            switch_dir,
        ],
    )
    .with_cwd(install_dir)
}

pub fn build_opam_env_command(opam: &str, context: &PluginContext) -> CommandPlan {
    let switch_dir = context
        .tool_dir
        .real_path_string()
        .unwrap_or_else(|| context.tool_dir.to_string());

    CommandPlan::new(
        opam,
        vec![
            "env".into(),
            "--sexp".into(),
            "--root".into(),
            opam_root_dir(&context.tool_dir)
                .real_path_string()
                .unwrap_or_else(|| opam_root_dir(&context.tool_dir).to_string()),
            "--switch".into(),
            switch_dir,
            "--set-root".into(),
            "--set-switch".into(),
        ],
    )
    .with_cwd(&context.tool_dir)
}

pub fn parse_opam_env_sexp(data: &str) -> Vec<(String, String)> {
    static PAIR_PATTERN: OnceLock<Regex> = OnceLock::new();
    let pattern = PAIR_PATTERN
        .get_or_init(|| Regex::new(r#"\(\s*"([^"]+)"\s*"((?:\\.|[^"])*)"\s*\)"#).unwrap());

    pattern
        .captures_iter(data)
        .map(|caps| {
            let value = caps[2].replace(r#"\""#, "\"").replace(r#"\\"#, "\\");

            (caps[1].to_owned(), value)
        })
        .collect()
}

pub fn split_tool_paths(
    path_value: &str,
    tool_dir: &VirtualPath,
    env: &HostEnvironment,
) -> Vec<PathBuf> {
    let separator = if env.os.is_windows() { ';' } else { ':' };
    let tool_root = tool_dir
        .real_path()
        .unwrap_or_else(|| tool_dir.to_path_buf());
    let mut paths = Vec::new();

    for entry in path_value
        .split(separator)
        .filter(|entry| !entry.is_empty())
    {
        let entry = PathBuf::from(entry);

        if !entry.starts_with(&tool_root) || paths.contains(&entry) {
            continue;
        }

        paths.push(entry);
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use proto_pdk::HostLibc;

    const FIXTURE_OCAML_VERSION: &str = "5.4.1";

    fn host_env(os: HostOS, arch: HostArch) -> HostEnvironment {
        HostEnvironment {
            arch,
            ci: false,
            libc: HostLibc::Gnu,
            os,
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
    fn resolves_opam_release_assets_for_supported_targets() -> AnyResult<()> {
        let version = Version::parse("2.5.0")?;
        let cases = [
            (
                host_env(HostOS::Linux, HostArch::X64),
                "opam-2.5.0-x86_64-linux",
            ),
            (
                host_env(HostOS::Linux, HostArch::Arm64),
                "opam-2.5.0-arm64-linux",
            ),
            (
                host_env(HostOS::MacOS, HostArch::Arm64),
                "opam-2.5.0-arm64-macos",
            ),
            (
                host_env(HostOS::Windows, HostArch::X64),
                "opam-2.5.0-x86_64-windows.exe",
            ),
        ];

        for (env, expected) in cases {
            assert_eq!(opam_asset_name(&env, &version)?, expected);
        }

        Ok(())
    }

    #[test]
    fn builds_non_interactive_init_command_for_windows() {
        let plan = build_opam_init_command(
            &real_tool_path("bin/opam.exe"),
            &tool_dir(),
            &host_env(HostOS::Windows, HostArch::X64),
        );

        assert_eq!(plan.command, real_tool_path("bin/opam.exe"));
        assert!(plan.args.contains(&"--cygwin-internal-install".into()));
        assert_eq!(plan.cwd, Some(tool_dir()));
    }

    #[test]
    fn builds_switch_create_command_for_local_switch() -> AnyResult<()> {
        let expected_tool_dir = real_tool_dir();
        let expected_opam_root = real_tool_path(".opam-root");

        let plan = build_switch_create_command(
            &real_tool_path("bin/opam"),
            &tool_dir(),
            &VersionSpec::parse(FIXTURE_OCAML_VERSION)?,
        );

        assert_eq!(plan.command, real_tool_path("bin/opam"));
        assert_eq!(
            plan.args,
            vec![
                "switch".into(),
                "create".into(),
                expected_tool_dir,
                format!("ocaml-base-compiler.{FIXTURE_OCAML_VERSION}"),
                "--yes".into(),
                "--root".into(),
                expected_opam_root,
            ],
        );
        assert_eq!(plan.cwd, Some(tool_dir()));

        Ok(())
    }

    #[test]
    fn builds_opam_env_command_with_explicit_root_and_switch() -> AnyResult<()> {
        let expected_tool_dir = real_tool_dir();
        let expected_opam_root = real_tool_path(".opam-root");

        let context = PluginContext {
            proto_version: Some(Version::new(0, 55, 3)),
            temp_dir: VirtualPath::Real(PathBuf::from("/tmp/proto-ocaml")),
            tool_dir: tool_dir(),
            version: VersionSpec::parse(FIXTURE_OCAML_VERSION)?,
            working_dir: VirtualPath::Real(PathBuf::from("/workspace")),
        };

        let plan = build_opam_env_command(&real_tool_path("bin/opam"), &context);

        assert_eq!(
            plan.args,
            vec![
                "env".into(),
                "--sexp".into(),
                "--root".into(),
                expected_opam_root,
                "--switch".into(),
                expected_tool_dir,
                "--set-root".into(),
                "--set-switch".into(),
            ],
        );

        Ok(())
    }

    #[test]
    fn parses_opam_env_sexp_pairs_with_escaped_values() {
        let values = parse_opam_env_sexp(&format!(
            r#"(("OPAMROOT" "{}/.opam-root")
                ("CAML_LD_LIBRARY_PATH" "/tmp/with\\backslash")
                ("MERLIN_LOG" "quoted:\"value\""))"#,
            real_tool_dir(),
        ));

        assert_eq!(
            values,
            vec![
                ("OPAMROOT".into(), real_tool_path(".opam-root"),),
                ("CAML_LD_LIBRARY_PATH".into(), "/tmp/with\\backslash".into()),
                ("MERLIN_LOG".into(), "quoted:\"value\"".into()),
            ],
        );
    }

    #[test]
    fn keeps_tool_local_paths_from_opam_env_and_dedupes_them() {
        let env = host_env(HostOS::Linux, HostArch::Arm64);
        let paths = split_tool_paths(
            &format!("{0}/bin:{0}/_opam/bin:/usr/bin:{0}/bin", real_tool_dir(),),
            &tool_dir(),
            &env,
        );

        assert_eq!(
            paths,
            vec![
                PathBuf::from(real_tool_path("bin")),
                PathBuf::from(real_tool_path("_opam/bin")),
            ],
        );
    }
}
