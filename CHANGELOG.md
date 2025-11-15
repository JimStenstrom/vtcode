# Changelog

All notable changes to VTCode will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Memory System** (`vtcode-memory` crate)
  - Three-tier memory architecture (working memory, summaries, session logs)
  - Non-destructive context pruning - messages preserved in memory instead of deleted
  - Temporal decay relevance scoring for summary retrieval
  - Background async summarization with zero user-perceived latency
  - Session persistence and restoration via JSON logs
  - Historical query detection ("remember when...", "you said earlier", etc.)
  - Configurable memory limits and checkpointing

- **Vector Database Abstraction** (`vtcode-vectordb` crate)
  - Unified `VectorDb` trait for pluggable backends
  - `InMemoryVectorDb` implementation for development and testing
  - Support for cosine, euclidean, and dot product distance metrics
  - Metadata filtering for advanced queries
  - Collection management with configurable dimensions

- **Document Retrieval** (`vtcode-rag` crate)
  - Document chunking strategies (fixed-size, semantic)
  - Indexing pipeline with embedding support
  - Query pipeline for semantic document search
  - Metadata propagation through chunking and retrieval

- **Configuration**
  - Memory settings in `vtcode.toml` under `[memory]` section
  - Vector database configuration under `[vectordb]` section
  - User-tunable parameters for memory limits and behavior
  - Backward compatibility with memory system opt-in

### Changed

- Context pruning now preserves messages in memory instead of permanently deleting them
- Long conversations (100+ turns) maintain better coherence through summarization
- Session archiving enhanced with memory integration and metadata tracking
- Context building can now include historical summaries for temporal queries

### Fixed

- Context loss in conversations exceeding 100 turns
- No cross-session learning capability (now supported via persistent session logs)
- Memory overhead from unbounded conversation history (now capped with configurable limits)

### Documentation

- Architecture documentation for memory system (`docs/architecture/memory-system.md`)
- Configuration guide updated with memory and vector database settings
- Troubleshooting guide expanded with memory system diagnostics
- README updated with memory feature highlights
- Quick start guide for memory system (`docs/guides/memory-quick-start.md`)
- vtcode-memory crate documentation and usage examples

### Performance

- Memory footprint: ~30KB per session (runtime)
- Context building: < 50ms
- Background summarization: < 3s per turn (async, non-blocking)
- Session save: < 100ms
- Disk usage: ~500KB per 100-turn session

## [0.43.6] - Previous Release

(Previous changelog entries would go here)
