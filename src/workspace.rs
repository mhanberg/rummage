use enum_dispatch::enum_dispatch;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDefinition {
    pub name: String,
    pub conditions: Vec<WorkspaceConditionEnum>,
}

#[enum_dispatch]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceConditionEnum {
    HasAnyFileCondition,
    HasAllFilesCondition,
    MissingAnyFileCondition,
    MissingAllFilesCondition,
    NullCondition,
}

#[enum_dispatch(WorkspaceConditionEnum)]
pub trait WorkspaceCondition {
    fn meets_condition(&self, path: &Path) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HasAnyFileCondition {
    pub files: Vec<String>,
}

impl WorkspaceCondition for HasAnyFileCondition {
    fn meets_condition(&self, path: &Path) -> bool {
        for file in &self.files {
            if path.join(file).exists() {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HasAllFilesCondition {
    pub files: Vec<String>,
}

impl WorkspaceCondition for HasAllFilesCondition {
    fn meets_condition(&self, path: &Path) -> bool {
        for file in &self.files {
            if !path.join(file).exists() {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingAnyFileCondition {
    pub files: Vec<String>,
}

impl WorkspaceCondition for MissingAnyFileCondition {
    fn meets_condition(&self, path: &Path) -> bool {
        for file in &self.files {
            if !path.join(file).exists() {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingAllFilesCondition {
    pub files: Vec<String>,
}

impl WorkspaceCondition for MissingAllFilesCondition {
    fn meets_condition(&self, path: &Path) -> bool {
        for file in &self.files {
            if path.join(file).exists() {
                return false;
            }
        }
        true
    }
}

/// A condition that always returns true, used as a default condition if no others
/// are specified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NullCondition {}

impl WorkspaceCondition for NullCondition {
    fn meets_condition(&self, _path: &Path) -> bool {
        true
    }
}

#[inline(always)]
pub fn path_meets_workspace_conditions(path: &Path, conditions: &[WorkspaceConditionEnum]) -> bool {
    conditions.iter().all(|c| c.meets_condition(path))
}

#[inline(always)]
pub fn get_workspace_type_for_path<'a>(
    path: &Path,
    workspace_definitions: &'a [WorkspaceDefinition],
) -> Option<&'a str> {
    for workspace_definition in workspace_definitions {
        if path_meets_workspace_conditions(path, &workspace_definition.conditions) {
            return Some(&workspace_definition.name);
        }
    }
    None
}

