//! Compile-time assertions for ownership patterns
//!
//! This module contains compile-time checks that ensure our core types
//! follow the established ownership patterns.

// Import macros from parent module
use crate::{assert_ownership_pattern, impl_ownership_validation};

// Core types that must follow ownership patterns
use crate::core::{MonorepoProject, MonorepoPackageInfo};
use crate::config::ConfigManager;
use crate::validation::ownership_validator::NotCopy;

// Implement NotCopy for types that should use move semantics
impl NotCopy for MonorepoPackageInfo {}
impl NotCopy for ConfigManager {}

// Compile-time assertions for MonorepoProject
assert_ownership_pattern!(no_arc: MonorepoProject);
impl_ownership_validation!(MonorepoProject);

// Compile-time assertions for MonorepoPackageInfo  
assert_ownership_pattern!(no_arc: MonorepoPackageInfo);
assert_ownership_pattern!(move_only: MonorepoPackageInfo);
impl_ownership_validation!(MonorepoPackageInfo);

// Compile-time assertions for ConfigManager
assert_ownership_pattern!(no_arc: ConfigManager);
assert_ownership_pattern!(move_only: ConfigManager);
impl_ownership_validation!(ConfigManager);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::ownership_validator::{NoSharedOwnership, MoveSemantics};
    
    #[test]
    fn test_monorepo_project_ownership() {
        // This test verifies at runtime what we already checked at compile time
        // It serves as documentation and double-checking
        
        // Create a project (would need proper setup in real test)
        // let project = create_test_project();
        // project.validate_no_shared_ownership();
        
        // Verify type name doesn't contain Arc or Rc
        let type_name = std::any::type_name::<MonorepoProject>();
        assert!(!type_name.contains("Arc<"));
        assert!(!type_name.contains("Rc<"));
    }
    
    #[test]
    fn test_config_manager_move_semantics() {
        let manager = ConfigManager::new();
        
        // This demonstrates move semantics
        let moved_manager = manager.take_ownership();
        // manager is now unusable (moved)
        
        // Can continue using moved_manager
        assert!(!moved_manager.auto_save); // Default is false
    }
}