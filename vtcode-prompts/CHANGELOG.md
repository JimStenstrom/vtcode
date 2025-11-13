# Changelog

All notable changes to vtcode-prompts will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.43.6] - 2025-01-13

### Added
- Initial extraction of prompts subsystem from vtcode-core
- `SystemPromptConfig` for configuring prompt generation
- `PromptContext` for adding contextual information to prompts
- `PromptTemplates` for reusable prompt components
- `SystemPromptGenerator` for composing complete system prompts
- Custom prompt system with variable substitution ($1, $NAME, $ARGUMENTS)
- `CustomPromptRegistry` for loading and managing custom prompts
- `BuiltinDocs` for embedded documentation support
- Support for three prompt variants: default, lightweight, and specialized
- Comprehensive test suite with unit and integration tests
- Example programs demonstrating usage patterns
- Full documentation in README.md

### Features
- Provider-agnostic design - returns raw prompt strings
- Minimal dependencies for maximum reusability
- Support for YAML frontmatter in custom prompts
- File size limits for custom prompts
- Duplicate prompt detection
- Shell-style argument parsing with quotes support
- Template variable escaping ($$)

### Testing
- Unit tests for all major components
- Integration tests for complete workflows
- Test coverage for error cases and edge conditions
- Examples that double as documentation and smoke tests

### Documentation
- Comprehensive README with usage examples
- Inline documentation for all public APIs
- Three example programs showing different use cases
- Integration guide for different LLM providers

## [Unreleased]

### Planned
- Build script to embed built-in prompts and documentation
- Additional built-in prompt templates
- More sophisticated template engine
- Prompt versioning and migration support
- Prompt template validation
- Performance optimizations for large prompt registries
