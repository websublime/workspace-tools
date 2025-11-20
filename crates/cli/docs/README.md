# CLI Implementation Plans

This directory contains detailed implementation plans for CLI features that are defined but not yet implemented.

---

## 📋 Overview

During a comprehensive CLI audit (2025-01-20), we identified **4 arguments that are defined in the CLI but not implemented**. This directory contains detailed implementation plans for each missing feature.

---

## 📁 Implementation Plans

### 1. [Prerelease Versions](./PRE-RELEASE-PLAN.md)
**Status**: Planning Complete  
**Priority**: High  
**Estimated Effort**: 3-5 days

**Missing Argument**: `workspace bump --prerelease <TAG>`

**Problem**: The `--prerelease` flag exists but is completely ignored. Users cannot create prerelease versions (alpha, beta, rc).

**Features**:
- Create prerelease versions: `1.2.3` → `1.3.0-beta.0`
- Increment prerelease: `1.3.0-beta.0` → `1.3.0-beta.1`
- Promote to stable: `1.3.0-rc.1` → `1.3.0`
- Smart changeset archival (don't archive prereleases)
- Works with any workflow (GitHub Flow, Gitflow, custom)

---

### 2. [Package Filtering](./PACKAGES-FILTER-PLAN.md)
**Status**: Planning Complete  
**Priority**: High  
**Estimated Effort**: 2-3 days

**Missing Argument**: `workspace bump --packages <LIST>`

**Problem**: The `--packages` flag exists but is ignored. Users cannot bump only specific packages.

**Features**:
- Selective package bumping: `--packages @org/core,@org/utils`
- Works with Independent and Unified strategies
- Emergency hotfix support (single package)
- Staged releases (frontend first, backend later)
- Optional dependency inclusion mode

---

### 3. [Registry Override](./REGISTRY-OVERRIDE-PLAN.md)
**Status**: Planning Complete  
**Priority**: Medium  
**Estimated Effort**: 0.5-1 day

**Missing Argument**: `workspace upgrade check --registry <URL>`

**Problem**: The `--registry` flag exists but is ignored. Users cannot check upgrades against custom registries.

**Features**:
- Custom registry URL: `--registry https://custom.com`
- Private/corporate registry support
- Local registry testing (Verdaccio)
- URL validation and normalization
- Clear logging of registry in use

---

### 4. [Backup Control](./BACKUP-CONTROL-PLAN.md)
**Status**: Planning Complete  
**Priority**: Low  
**Estimated Effort**: 0.5-1 day

**Missing Argument**: `workspace upgrade apply --no-backup`

**Problem**: The `--no-backup` flag exists but is ignored. Backups are always created regardless.

**Features**:
- Skip backup creation for faster upgrades
- CI/CD optimization (no disk space waste)
- Warning when backups disabled
- Recommends Git as alternative
- Performance improvements (50-67% faster)

---

## 🎯 Implementation Priority

### High Priority
1. **Prerelease Versions** - Critical for release workflows
2. **Package Filtering** - Important for emergency hotfixes

### Medium Priority
3. **Registry Override** - Nice to have for enterprise users

### Low Priority
4. **Backup Control** - Minor optimization for CI/CD

---

## 📊 Current Implementation Status

| Feature | Argument | Status | Implementation |
|---------|----------|--------|----------------|
| Prerelease Versions | `--prerelease` | ❌ Not Implemented | 0% |
| Package Filtering | `--packages` | ❌ Not Implemented | 0% |
| Registry Override | `--registry` | ❌ Not Implemented | 0% |
| Backup Control | `--no-backup` | ❌ Not Implemented | 0% |

**Total**: 4 out of 96 arguments (95.8% implemented)

---

## 📝 Plan Document Structure

Each implementation plan follows this structure:

1. **Executive Summary**
   - Problem statement
   - Solution overview
   - Key features

2. **Current System State**
   - What exists
   - What's missing
   - Infrastructure available

3. **Problem Analysis**
   - Core questions
   - Design decisions
   - Trade-offs

4. **Proposed Architecture**
   - High-level flow
   - Component design
   - Data structures

5. **Implementation Details**
   - Code changes required
   - Function signatures
   - Integration points

6. **Use Cases**
   - Real-world scenarios
   - Command examples
   - Expected outcomes

7. **Implementation Checklist**
   - Phase-by-phase tasks
   - Testing requirements
   - Documentation needs

8. **Risks and Mitigations**
   - Potential issues
   - Mitigation strategies
   - Impact assessment

9. **Summary**
   - Key features recap
   - Success metrics
   - Timeline estimate

---

## 🚀 Getting Started

To implement any of these features:

1. **Read the plan** - Review the detailed implementation plan
2. **Understand scope** - Check estimated effort and dependencies
3. **Create task** - Create tracking issue/story
4. **Follow checklist** - Use the implementation checklist as guide
5. **Test thoroughly** - All phases include comprehensive testing
6. **Document** - Update relevant documentation as you go

---

## 🔗 Related Documentation

- [CLI Specification](../SPEC.md)
- [Main README](../../../README.md)
- [Package Tools SPEC](../../pkg/SPEC.md)
- [Git Tools SPEC](../../git/SPEC.md)

---

## 📫 Questions?

If you have questions about any of these implementation plans:

1. Review the detailed plan document
2. Check the "Problem Analysis" section for design rationale
3. Look at "Use Cases" for practical examples
4. Consult with the team for architectural decisions

---

**Last Updated**: 2025-01-20  
**Audit Date**: 2025-01-20  
**Next Review**: After implementing each feature
