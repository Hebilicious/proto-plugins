use crate::opam::*;
use crate::version_files::normalize_ocaml_version;
#[cfg(target_arch = "wasm32")]
use crate::version_files::parse_version_file as parse_ecosystem_version_file;
#[cfg(target_arch = "wasm32")]
use anyhow::anyhow;
#[cfg(target_arch = "wasm32")]
use extism_pdk::*;
use proto_pdk::*;
#[cfg(target_arch = "wasm32")]
use starbase_utils::fs;
use std::collections::HashSet;
use std::path::PathBuf;

#[cfg(target_arch = "wasm32")]
#[host_fn]
extern "ExtismHost" {
    fn exec_command(input: Json<ExecCommandInput>) -> Json<ExecCommandOutput>;
}

pub static NAME: &str = "OCaml";

#[cfg(target_arch = "wasm32")]
fn execute_plan(
    plan: CommandPlan,
    set_executable: bool,
    stream: bool,
) -> AnyResult<ExecCommandOutput> {
    let result = unsafe {
        exec_command(Json(ExecCommandInput {
            command: plan.command,
            args: plan.args,
            cwd: plan.cwd,
            env: plan.env.into_iter().collect(),
            set_executable,
            stream,
            ..ExecCommandInput::default()
        }))?
    }
    .0;

    if result.exit_code != 0 {
        return Err(anyhow!(
            "Command failed: {}",
            if result.stderr.is_empty() {
                result.stdout.trim().to_owned()
            } else {
                result.stderr.trim().to_owned()
            }
        ));
    }

    Ok(result)
}

pub fn ensure_supported_target(env: &HostEnvironment) -> AnyResult<()> {
    check_supported_os_and_arch(
        NAME,
        env,
        permutations![
            HostOS::Linux => [HostArch::X64, HostArch::Arm64],
            HostOS::MacOS => [HostArch::X64, HostArch::Arm64],
            HostOS::Windows => [HostArch::X64],
        ],
    )
}

pub fn normalize_ocaml_tag(tag: &str) -> Option<String> {
    let tag = tag.trim().trim_start_matches('v');

    if !tag
        .split('.')
        .all(|part| !part.is_empty() && part.chars().all(|char| char.is_ascii_digit()))
    {
        return None;
    }

    let normalized = normalize_ocaml_version(tag);

    (normalized.matches('.').count() == 2).then_some(normalized)
}

pub fn build_register_tool_output() -> AnyResult<RegisterToolOutput> {
    Ok(RegisterToolOutput {
        name: NAME.into(),
        type_of: PluginType::Language,
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
        .filter_map(|tag| normalize_ocaml_tag(&tag))
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
        files: vec![".ocaml-version".into(), "dune-project".into()],
        ignore: vec!["_build".into()],
    }
}

pub fn build_locate_executables_output(env: &HostEnvironment) -> LocateExecutablesOutput {
    LocateExecutablesOutput {
        exes: [
            (
                "opam".into(),
                ExecutableConfig::new(env.os.get_exe_name("bin/opam")),
            ),
            (
                "ocaml".into(),
                ExecutableConfig::new_primary(env.os.get_exe_name("_opam/bin/ocaml")),
            ),
            (
                "ocamlc".into(),
                ExecutableConfig::new(env.os.get_exe_name("_opam/bin/ocamlc")),
            ),
            (
                "ocamlopt".into(),
                ExecutableConfig::new(env.os.get_exe_name("_opam/bin/ocamlopt")),
            ),
            (
                "ocamldep".into(),
                ExecutableConfig::new(env.os.get_exe_name("_opam/bin/ocamldep")),
            ),
            (
                "dune".into(),
                ExecutableConfig::new(env.os.get_exe_name("_opam/bin/dune")),
            ),
        ]
        .into_iter()
        .collect(),
        exes_dirs: vec![PathBuf::from("bin"), opam_switch_bin_dir()],
        ..LocateExecutablesOutput::default()
    }
}

pub fn build_activate_environment_output(
    sexp: &str,
    tool_dir: &VirtualPath,
    env: &HostEnvironment,
) -> ActivateEnvironmentOutput {
    let mut output = ActivateEnvironmentOutput::default();

    for (key, value) in parse_opam_env_sexp(sexp) {
        if key == "PATH" {
            output.paths = split_tool_paths(&value, tool_dir, env);
        } else {
            output.env.insert(key, value);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use proto_pdk::HostLibc;

    const FIXTURE_LATEST_VERSION: &str = "5.4.1";
    const FIXTURE_PREVIOUS_VERSION: &str = "5.4.0";

    fn host_env(os: HostOS, arch: HostArch) -> HostEnvironment {
        HostEnvironment {
            arch,
            ci: false,
            libc: HostLibc::Gnu,
            os,
            home_dir: VirtualPath::Real(PathBuf::from("/home/tester")),
        }
    }

    #[test]
    fn normalizes_supported_ocaml_tags_only() {
        assert_eq!(normalize_ocaml_tag("v4.08.1"), Some("4.8.1".into()));
        assert_eq!(
            normalize_ocaml_tag(FIXTURE_LATEST_VERSION),
            Some(FIXTURE_LATEST_VERSION.into()),
        );
        assert_eq!(normalize_ocaml_tag("5.4"), None);
        assert_eq!(normalize_ocaml_tag("trunk"), None);
        assert_eq!(
            normalize_ocaml_tag(&format!("{FIXTURE_LATEST_VERSION}+flambda")),
            None,
        );
    }

    #[test]
    fn builds_versions_output_with_stable_alias_and_deduped_versions() -> AnyResult<()> {
        let output = build_load_versions_output(vec![
            "4.08.1".into(),
            "4.8.1".into(),
            format!("v{FIXTURE_PREVIOUS_VERSION}"),
            FIXTURE_LATEST_VERSION.into(),
            "trunk".into(),
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
                VersionSpec::parse("4.8.1")?,
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
    fn declares_ecosystem_file_precedence_and_build_ignore() {
        let output = build_detect_version_output();

        assert_eq!(output.files, vec![".ocaml-version", "dune-project"]);
        assert_eq!(output.ignore, vec!["_build"]);
    }

    #[test]
    fn exposes_expected_executables_and_primary_binary() {
        let output = build_locate_executables_output(&host_env(HostOS::Linux, HostArch::Arm64));

        assert_eq!(
            output.exes_dirs,
            vec![PathBuf::from("bin"), PathBuf::from("_opam/bin")]
        );
        assert_eq!(
            output.exes["ocaml"],
            ExecutableConfig::new_primary("_opam/bin/ocaml"),
        );
        assert_eq!(output.exes["opam"], ExecutableConfig::new("bin/opam"));
        assert_eq!(output.exes["dune"], ExecutableConfig::new("_opam/bin/dune"));
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
        "https://github.com/ocaml/ocaml",
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
pub fn native_install(
    Json(input): Json<NativeInstallInput>,
) -> FnResult<Json<NativeInstallOutput>> {
    let env = get_host_environment()?;
    ensure_supported_target(&env)?;

    fs::create_dir_all(input.install_dir.join(OPAM_BIN_DIR).to_path_buf())?;

    let opam_version = opam_release_version();
    let opam_url = opam_download_url(&env, &opam_version)?;
    let opam_bytes = fetch_bytes(opam_url)?;
    let opam_bin = input.install_dir.join(opam_install_bin(&env));

    fs::write_file(opam_bin.to_path_buf(), opam_bytes)?;

    let opam_command = opam_bin.real_path_string().ok_or_else(|| {
        PluginError::Message(format!("Failed to resolve {} to a real path", opam_bin))
    })?;

    execute_plan(
        build_opam_init_command(&opam_command, &input.install_dir, &env),
        !env.os.is_windows(),
        true,
    )?;

    execute_plan(
        build_switch_create_command(&opam_command, &input.install_dir, &input.context.version),
        !env.os.is_windows(),
        true,
    )?;

    execute_plan(
        build_dune_install_command(&opam_command, &input.install_dir),
        !env.os.is_windows(),
        true,
    )?;

    Ok(Json(NativeInstallOutput {
        installed: true,
        ..NativeInstallOutput::default()
    }))
}

#[cfg(target_arch = "wasm32")]
#[plugin_fn]
pub fn native_uninstall(
    Json(input): Json<NativeUninstallInput>,
) -> FnResult<Json<NativeUninstallOutput>> {
    if input.uninstall_dir.exists() {
        fs::remove_dir_all(input.uninstall_dir.to_path_buf())?;
    }

    Ok(Json(NativeUninstallOutput {
        uninstalled: true,
        ..NativeUninstallOutput::default()
    }))
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

#[cfg(target_arch = "wasm32")]
#[plugin_fn]
pub fn activate_environment(
    Json(input): Json<ActivateEnvironmentInput>,
) -> FnResult<Json<ActivateEnvironmentOutput>> {
    let env = get_host_environment()?;
    let opam_bin = opam_executable_path(&input.context.tool_dir, &env);
    let opam_command = opam_bin
        .real_path_string()
        .unwrap_or_else(|| opam_bin.to_string());

    let result = execute_plan(
        build_opam_env_command(&opam_command, &input.context),
        !env.os.is_windows(),
        false,
    )?;

    Ok(Json(build_activate_environment_output(
        &result.stdout,
        &input.context.tool_dir,
        &env,
    )))
}
