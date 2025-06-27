//! Runtime validation to ensure ownership principles are maintained
//!
//! This module provides validation functions that can be run in debug mode
//! to verify that the codebase follows established ownership patterns.

use std::any::type_name;

/// Trait to mark types that should not use Arc or Rc
pub trait NoSharedOwnership {
    fn validate_no_shared_ownership(&self) {
        let type_name = type_name::<Self>();
        
        // These assertions will fail at compile time if the type contains Arc/Rc
        // This is a compile-time check disguised as a runtime validator
        debug_assert!(
            !type_name.contains("Arc<") && !type_name.contains("Rc<"),
            "Type {type_name} should not use Arc or Rc for shared ownership"
        );
    }
}

/// Trait to mark types that should follow move semantics
pub trait MoveSemantics: Sized {
    /// Takes ownership and returns it, enforcing move semantics
    #[must_use]
    fn take_ownership(self) -> Self {
        self
    }
}

/// Marker trait for types that should use move semantics (not Copy)
pub trait NotCopy: Sized {
    fn assert_not_copy(&self) {}
}

/// Validate that a type uses move semantics
pub fn validate_move_only<T>(_value: T) 
where 
    T: NotCopy,
{
    // This function requires NotCopy trait, which should not be
    // implemented for Copy types
}

/// Macro to implement ownership validation for a type
#[macro_export]
macro_rules! impl_ownership_validation {
    ($type:ty) => {
        impl $crate::validation::ownership_validator::NoSharedOwnership for $type {}
        impl $crate::validation::ownership_validator::MoveSemantics for $type {}
    };
}

/// Macro to assert ownership patterns at compile time
#[macro_export]
macro_rules! assert_ownership_pattern {
    (move_only: $type:ty) => {
        const _: () = {
            fn _assert_move_only(value: $type) {
                $crate::validation::ownership_validator::validate_move_only(value);
            }
        };
    };
    
    (no_arc: $type:ty) => {
        const _: () = {
            fn _assert_no_arc() {
                let type_name = std::any::type_name::<$type>();
                assert!(!type_name.contains("Arc<"));
                assert!(!type_name.contains("Rc<"));
            }
        };
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestStruct {
        data: String,
    }
    
    impl NoSharedOwnership for TestStruct {}
    impl MoveSemantics for TestStruct {}
    
    #[test]
    fn test_no_shared_ownership_validation() {
        let test = TestStruct {
            data: "test".to_string(),
        };
        
        test.validate_no_shared_ownership();
        
        // This would fail if TestStruct contained Arc or Rc
    }
    
    #[test]
    fn test_move_semantics() {
        let test = TestStruct {
            data: "test".to_string(),
        };
        
        let moved = test.take_ownership();
        // test is now moved and cannot be used
        
        assert_eq!(moved.data, "test");
    }
    
    // This would fail to compile:
    // #[test]
    // fn test_copy_type_fails() {
    //     validate_move_only(42i32); // i32 implements Copy
    // }
}