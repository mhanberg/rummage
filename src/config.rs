use crate::workspace::{
    HasAnyFileCondition, MissingAllFilesCondition, MissingAnyFileCondition, NullCondition,
    WorkspaceConditionEnum, WorkspaceDefinition,
};
use anyhow::{Context, Result};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
struct WorkspaceDefinitionConfig {
    /// Name for the workspace type defined by the list item.
    ///
    /// This name corresponds to the `RUMMAGE_TYPE` environment variable that will be set inside a session.
    pub name: String,

    /// List of files for which at least one must be present in a directory to be considered a workspace of this type.
    ///
    /// If unset, this constraint is simply ignored.
    ///
    /// For example if the list is `["requirements.txt", "Pipfile", "pyproject.toml", "poetry.lock", "setup.py"]`, a
    /// directory not containing *any* of those files cannot match this workspace definition.
    pub has_any_file: Option<Vec<String>>,

    /// List of files for which all must be present in a directory to be considered a workspace of this type.
    ///
    /// If unset, this constraint is simply ignored.
    ///
    /// For example, if the list is `["flake.nix", ".envrc"]`, only directories with *both* files present can match
    /// this workspace definition.
    pub has_all_files: Option<Vec<String>>,

    /// List of files for which at least one must be missing in a directory to be considered a workspace of this type.
    ///
    /// If unset, this constraint is simply ignored.
    ///
    /// For example, if the list is `["node_modules", "target"]`, directories containing *both* `node_modules` and `target`
    /// cannot match this workspace definition.
    pub missing_any_file: Option<Vec<String>>,

    /// List of files for which all must be missing in a directory to be considered a workspace of this type.
    ///
    /// If unset, this constraint is simply ignored.
    ///
    /// For example, if the list is `["node_modules", "target"]`, directories containing *either* `node_modules` or `target`
    /// cannot match this workspace definition.
    pub missing_all_files: Option<Vec<String>>,
}

impl From<WorkspaceDefinitionConfig> for WorkspaceDefinition {
    fn from(config: WorkspaceDefinitionConfig) -> Self {
        let mut conditions = Vec::<WorkspaceConditionEnum>::new();

        if let Some(has_any_file) = config.has_any_file {
            if !has_any_file.is_empty() {
                let condition = HasAnyFileCondition {
                    files: has_any_file,
                };
                conditions.push(condition.into());
            }
        }

        if let Some(has_all_files) = config.has_all_files {
            if !has_all_files.is_empty() {
                let condition = HasAnyFileCondition {
                    files: has_all_files,
                };
                conditions.push(condition.into());
            }
        }

        if let Some(missing_any_file) = config.missing_any_file {
            if !missing_any_file.is_empty() {
                let condition = MissingAnyFileCondition {
                    files: missing_any_file,
                };
                conditions.push(condition.into());
            }
        }

        if let Some(missing_all_files) = config.missing_all_files {
            if !missing_all_files.is_empty() {
                let condition = MissingAllFilesCondition {
                    files: missing_all_files,
                };
                conditions.push(condition.into());
            }
        }

        if conditions.is_empty() {
            let condition = NullCondition {};
            conditions.push(condition.into());
        }

        WorkspaceDefinition {
            name: config.name,
            conditions,
        }
    }
}

fn default_search_paths() -> Vec<String> {
    vec!["~".into()]
}

fn default_workspace_definitions() -> Vec<WorkspaceDefinitionConfig> {
    vec![WorkspaceDefinitionConfig {
        name: "default".into(),
        has_any_file: Some(vec![".git".into(), ".rummage.yaml".into()]),
        has_all_files: None,
        missing_any_file: None,
        missing_all_files: None,
    }]
}

const fn default_max_search_depth() -> usize {
    3
}

const fn default_session_name_path_components() -> usize {
    2
}

fn default_exclude_path_components() -> Vec<String> {
    vec![
        ".cache".into(),
        ".cargo".into(),
        ".git".into(),
        "__pycache__".into(),
        "node_modules".into(),
        "target".into(),
        "venv".into(),
    ]
}

fn default_follow_links() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RawRummageGlobal {
    /// List of directories to have rummage search for workspaces.
    ///
    /// If unset, defaults to `~` (shell expansion is supported).
    ///
    /// Be careful to not make your search paths overlap, e.g. if you include `~/projects` and `~/projects/foo/bar`
    /// with `max_search_depth: 3`, `~/projects/foo/bar` will be searched twice and results will be displayed twice
    /// in the picker. Generally it's easiest to just include the parent directory and increase `max_search_depth`
    /// if needed.
    #[serde(default = "default_search_paths")]
    search_paths: Vec<String>,

    /// List of configurations for workspaces.
    ///
    /// If unset, the default rummage workspace definition is any directory containing a `.git` file/directory or a
    /// `.rummage.yaml` file.
    ///
    /// When a directory is found that matches a workspace definition the first match, in order of appearance in
    /// this list and in setting the `RUMMAGE_TYPE` environment variable
    #[serde(default = "default_workspace_definitions")]
    workspace_definitions: Vec<WorkspaceDefinitionConfig>,

    /// Maximum depth to search for workspaces inside the `search_paths` directories.
    /// If unset, defaults to 3.
    #[serde(default = "default_max_search_depth")]
    max_search_depth: usize,

    /// Default number of components of the workspace directory to use for the created session name.
    /// If unset, defaults to 1.
    ///
    /// E.g. if you open a workspace at `/home/vinny/projects/foo/bar` and `session_name_path_components` is set to 1,
    /// The session name will be `bar`. If 2, `foo/bar`, etc.
    #[serde(default = "default_session_name_path_components")]
    session_name_path_components: usize,

    /// List of path components which will *exclude* a directory from being considered a workspace.
    /// If unset, defaults to an empty list.
    ///
    /// A common use case would be to exclude things like `node_modules`, `target`, `__pycache__`, etc.
    #[serde(default = "default_exclude_path_components")]
    exclude_path_components: Vec<String>,

    /// Whether to follow symbolic links when searching for workspaces.
    /// If unset, defaults to true.
    #[serde(default = "default_follow_links")]
    follow_links: bool,
}

impl Default for RawRummageGlobal {
    fn default() -> Self {
        // test case ensures this works
        RawRummageGlobal::from_str("").unwrap()
    }
}

impl RawRummageGlobal {
    pub fn schema() -> Result<String> {
        Ok(serde_json::to_string_pretty(&schema_for!(Self))?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RummageGlobal {
    pub search_paths: Vec<String>,
    pub exclude_path_components: Vec<String>,
    pub workspace_definitions: Vec<WorkspaceDefinition>,
    pub session_name_path_components: usize,
    pub max_search_depth: usize,
    pub follow_links: bool,
}

impl From<RawRummageGlobal> for RummageGlobal {
    fn from(raw_config: RawRummageGlobal) -> Self {
        // search paths are the only place we need to worry about shell expansion
        let search_paths: Vec<String> = raw_config
            .search_paths
            .iter()
            .map(|path| shellexpand::tilde(path).to_string())
            .collect();

        let exclude_path_components = raw_config.exclude_path_components;

        let workspace_definitions = raw_config
            .workspace_definitions
            .into_iter()
            .map(WorkspaceDefinition::from)
            .collect();

        Self {
            search_paths,
            exclude_path_components,
            workspace_definitions,
            max_search_depth: raw_config.max_search_depth,
            session_name_path_components: raw_config.session_name_path_components,
            follow_links: raw_config.follow_links,
        }
    }
}

impl TryFrom<&PathBuf> for RawRummageGlobal {
    type Error = anyhow::Error;

    fn try_from(path: &PathBuf) -> Result<Self> {
        let config = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config from path: {path:#?}"))?;
        let raw_config = RawRummageGlobal::from_str(&config)
            .with_context(|| "Failed to parse rummage config file.")?;
        Ok(raw_config)
    }
}

impl FromStr for RawRummageGlobal {
    type Err = anyhow::Error;

    fn from_str(config: &str) -> Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::from_str(config, config::FileFormat::Yaml))
            .build()
            .with_context(|| "Failed build configuration. You should never see this. I think.")?;

        let raw_config = settings
            .try_deserialize()
            .with_context(|| "Failed to deserialize rummage config.")?;
        Ok(raw_config)
    }
}

impl RummageGlobal {
    fn get_config_path() -> Result<Option<PathBuf>> {
        let config_file_name = format!("{}.yaml", clap::crate_name!());
        match std::env::var_os("RUMMAGE_CONFIG_FILE") {
            // if RUMMAGE_CONFIG_FILE is not set, search xdg dirs for config file as normal
            c if c.as_ref().unwrap_or(&OsString::default()).is_empty() => {
                let xdg_dirs = xdg::BaseDirectories::with_prefix(clap::crate_name!())
                    .with_context(|| "Failed to load XDG dirs.")?;
                let xdg_config_path = xdg_dirs.get_config_file(config_file_name);
                match xdg_config_path.exists() {
                    true => Ok(Some(xdg_config_path)),
                    false => Ok(None),
                }
            }
            // if we explicitly set the RUMMAGE_CONFIG_FILE, we should take it at face value and return the path here
            // which will cause an error later if it doesn't turn out to exist. This choice is made because it could
            // be a really annoying silent error if someone set the env var override somewhere, forgot, and changed it
            // vs its unlikely that many people would not understand where they need to put their config file and end
            // up confused why their settings aren't being picked up. ignoring a missing conf file lets the program run
            // without someone explicitly setting up any config
            Some(config_file_path) => Ok(Some(PathBuf::from(config_file_path))),
            _ => unreachable!(),
        }
    }

    pub fn load() -> Result<Self> {
        let raw_config = match RummageGlobal::get_config_path()? {
            Some(path) => RawRummageGlobal::try_from(&path)?,
            None => RawRummageGlobal::default(),
        };
        let config = RummageGlobal::from(raw_config);
        Ok(config)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use serial_test::serial;

    #[test]
    fn test_empty_config_is_valid() {
        let raw_config = RawRummageGlobal::from_str("").unwrap();
        let _ = RummageGlobal::from(raw_config);
    }

    #[test]
    fn test_invalid_config_key_is_error() {
        let raw_config = RawRummageGlobal::from_str("foo: bar");
        assert!(raw_config.is_err());
    }

    /// Make noise if we change which env var overrides the config file path or it breaks
    #[test]
    #[serial]
    fn test_get_config_path_env_var_override() {
        let orig_rummage = std::env::var_os("RUMMAGE_CONFIG_FILE");
        let config_file = "/tmp/rummage.yaml";
        std::env::set_var("RUMMAGE_CONFIG_FILE", config_file);

        let config_path = RummageGlobal::get_config_path().unwrap();
        assert_eq!(config_path, Some(PathBuf::from(config_file)));

        if let Some(rummage) = orig_rummage {
            std::env::set_var("RUMMAGE_CONFIG_FILE", rummage);
        } else {
            std::env::remove_var("RUMMAGE_CONFIG_FILE");
        }
    }

    #[test]
    #[serial]
    fn test_get_config_path_xdg_default_file_doesnt_exist() {
        let orig_rummage = std::env::var_os("RUMMAGE_CONFIG_FILE");
        let orig_home = std::env::var_os("HOME");
        let orig_xdg = std::env::var_os("XDG_CONFIG_HOME");
        std::env::remove_var("RUMMAGE_CONFIG_FILE");
        std::env::set_var("HOME", "/tmp");
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/.config");
        let config_path = RummageGlobal::get_config_path().unwrap();
        assert_eq!(
            config_path,
            None,
            //Some(PathBuf::from("/tmp/.config/rummage/rummage.yaml"))
        );

        if let Some(rummage) = orig_rummage {
            std::env::set_var("RUMMAGE_CONFIG_FILE", rummage);
        }
        if let Some(home) = orig_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
        if let Some(xdg) = orig_xdg {
            std::env::set_var("XDG_CONFIG_HOME", xdg);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
    }

    /// this could end up being a flaky test we'll see
    #[test]
    #[serial]
    fn test_get_config_path_xdg_default_file_exists() {
        let orig_rummage = std::env::var_os("RUMMAGE_CONFIG_FILE");
        let orig_home = std::env::var_os("HOME");
        let orig_xdg = std::env::var_os("XDG_CONFIG_HOME");
        std::env::remove_var("RUMMAGE_CONFIG_FILE");
        std::env::set_var("HOME", "/tmp");
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/.config");
        std::fs::create_dir_all("/tmp/.config/rummage").unwrap();
        std::fs::write("/tmp/.config/rummage/rummage.yaml", "").unwrap();
        let config_path = RummageGlobal::get_config_path().unwrap();
        assert_eq!(
            config_path,
            Some(PathBuf::from("/tmp/.config/rummage/rummage.yaml"))
        );

        if let Some(rummage) = orig_rummage {
            std::env::set_var("RUMMAGE_CONFIG_FILE", rummage);
        }
        if let Some(home) = orig_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
        if let Some(xdg) = orig_xdg {
            std::env::set_var("XDG_CONFIG_HOME", xdg);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
        let _ = std::fs::remove_file("/tmp/.config/rummage/rummage.yaml");
    }

    #[test]
    #[serial]
    fn test_get_config_path_empty_string_equals_unset() {
        let orig_rummage = std::env::var_os("RUMMAGE_CONFIG_FILE");
        let orig_home = std::env::var_os("HOME");
        let orig_xdg = std::env::var_os("XDG_CONFIG_HOME");
        std::env::remove_var("RUMMAGE_CONFIG_FILE");
        std::env::set_var("HOME", "/tmp");
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/.config");

        let unset_rummage_file_config_path = RummageGlobal::get_config_path().unwrap();

        std::env::set_var("RUMMAGE_CONFIG_FILE", "");
        let empty_rummage_file_config_path = RummageGlobal::get_config_path().unwrap();

        assert_eq!(
            unset_rummage_file_config_path,
            empty_rummage_file_config_path
        );

        if let Some(rummage) = orig_rummage {
            std::env::set_var("RUMMAGE_CONFIG_FILE", rummage);
        } else {
            std::env::remove_var("RUMMAGE_CONFIG_FILE");
        }
        if let Some(home) = orig_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
        if let Some(xdg) = orig_xdg {
            std::env::set_var("XDG_CONFIG_HOME", xdg);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
    }
}
