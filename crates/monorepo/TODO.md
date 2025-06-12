 # Missing pieces

- [ ] In changes module, engine.rs file on function evaluate_conditions are missing implementation.
- [ ] In changesets module, manager.rs file on function validate_changeset naming conventions/prefix and branches names are hardcoded
- [ ] In core module, project.rs file on functions refresh_packages and build_dependency_graph are missing implementation
- [ ] In core module, version.rs file on function calculate_execution_order needs improve in the implementation
- [ ] In hooks module, context.rs file on function has_changed_files_matching improve matching patterns with glob
- [ ] In hooks module, validator.rs file on functions check_packages_have_changesets and find_changeset_for_packages are missing integration, function get_branch_naming_patterns hardcoded values, function check_git_ref_exists needs ann integration
- [ ] In tasks module, checker.rs file on functions execute_custom_script and execute_custom_environment_checker evaluate if we can use command from standard crate, on functions has_dependency_changes_in_package and analyze_package_change_level evaluate if git can be used from git crate
- [ ] In workflows module, integration.rs file on function validate_dependency_consistency needs a real implementation
- [ ] In workflows module, progress.rs file on function add_substep needs a implementation for substeps tracking
- [ ] We will need to implement logging in every module and crate. we are missing a lot of logs in the code, and we need to define a standard for logging.

# Inconsistencias

- We still have lots of mixin structs declaration with implementation in the same file
- Use of re-export all like core::*, changesets::*, etc.
- Mixin re-exports in the middle of implementation files and even re-exporting different crates
- Types and implementations are not organize by feature or type, lot's of mixin concerns
- Some files are composed like dependency_graph.rs
- Evaluate what should be the final exports of the lib.rs file. Some are just public to be used inside of the crate. It could be used the pub(crate) visibility.
- Many tests are not covering the functionalities.