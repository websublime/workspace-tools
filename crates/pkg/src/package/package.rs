use std::{cell::RefCell, rc::Rc};

use semver::Version;

use crate::{
    Dependency, DependencyRegistry, DependencyResolutionError, Node, ResolutionResult, VersionError,
};

#[derive(Debug, Clone)]
pub struct Package {
    name: String,
    version: Rc<RefCell<Version>>,
    dependencies: Vec<Rc<RefCell<Dependency>>>,
}

impl Node for Package {
    type DependencyType = crate::Dependency;
    type Identifier = String;

    fn dependencies(&self) -> Vec<&Self::DependencyType> {
        // Since we use Rc<RefCell<Dependency>>, we can't return direct references
        // Instead, return empty vec and use dependencies_vec
        Vec::new()
    }

    fn dependencies_vec(&self) -> Vec<Self::DependencyType> {
        self.dependencies()
            .iter()
            .filter_map(|dep| {
                let borrowed = dep.borrow();
                match borrowed.fixed_version() {
                    Ok(version) => Dependency::new(borrowed.name(), &version.to_string()).ok(),
                    _ => None,
                }
            })
            .collect()
    }

    fn matches(&self, dependency: &Self::DependencyType) -> bool {
        self.name() == dependency.name() && dependency.version().matches(&self.version())
    }

    fn identifier(&self) -> Self::Identifier {
        self.name().to_string()
    }
}

impl Package {
    /// Create a new package with name, version, and optional dependencies
    pub fn new(
        name: &str,
        version: &str,
        dependencies: Option<Vec<Rc<RefCell<Dependency>>>>,
    ) -> Result<Self, VersionError> {
        let parsed_version = version.parse().map_err(VersionError::from)?;

        Ok(Self {
            name: name.to_string(),
            version: Rc::new(RefCell::new(parsed_version)),
            dependencies: dependencies.unwrap_or_default(),
        })
    }

    /// Create a new package using the dependency registry
    pub fn new_with_registry(
        name: &str,
        version: &str,
        dependencies: Option<Vec<(&str, &str)>>,
        registry: &mut DependencyRegistry,
    ) -> Result<Self, VersionError> {
        let deps = if let Some(dep_list) = dependencies {
            let mut deps_vec = Vec::new();
            for (dep_name, dep_version) in dep_list {
                let dep = registry.get_or_create(dep_name, dep_version)?;
                deps_vec.push(dep);
            }
            deps_vec
        } else {
            Vec::new()
        };

        Self::new(name, version, Some(deps))
    }

    /// Get the package name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the package version
    pub fn version(&self) -> Version {
        self.version.borrow().clone()
    }

    /// Get the package version as a string
    pub fn version_str(&self) -> String {
        self.version.borrow().to_string()
    }

    /// Update the package version
    pub fn update_version(&self, new_version: &str) -> Result<(), VersionError> {
        let parsed_version = new_version.parse().map_err(VersionError::from)?;
        *self.version.borrow_mut() = parsed_version;
        Ok(())
    }

    /// Get the package dependencies
    pub fn dependencies(&self) -> &[Rc<RefCell<Dependency>>] {
        &self.dependencies
    }

    /// Update a dependency version
    pub fn update_dependency_version(
        &self,
        dep_name: &str,
        new_version: &str,
    ) -> Result<(), DependencyResolutionError> {
        for dep in &self.dependencies {
            let name = dep.borrow().name().to_string();
            if name == dep_name && dep.borrow().update_version(new_version).is_ok() {
                return Ok(());
            }
        }

        Err(DependencyResolutionError::DependencyNotFound {
            name: dep_name.to_string(),
            package: self.name.clone(),
        })
    }

    /// Add a dependency to the package
    pub fn add_dependency(&mut self, dependency: Rc<RefCell<Dependency>>) {
        self.dependencies.push(dependency);
    }

    /// Update package dependencies based on resolution result
    pub fn update_dependencies_from_resolution(
        &self,
        resolution: &ResolutionResult,
    ) -> Result<Vec<(String, String, String)>, VersionError> {
        let mut updated_deps = Vec::new();

        for dep in &self.dependencies {
            let name = dep.borrow().name().to_string();
            if let Some(resolved_version) = resolution.resolved_versions.get(&name) {
                let current_version = dep.borrow().fixed_version()?.to_string();

                // Only update if the versions are different
                if current_version != *resolved_version
                    && !current_version.contains(resolved_version)
                {
                    dep.borrow().update_version(resolved_version)?;
                    updated_deps.push((name, current_version, resolved_version.clone()));
                }
            }
        }

        Ok(updated_deps)
    }
}
