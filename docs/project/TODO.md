can we wrap the TUI container text for both edges of terminals? It looks weird when the text just abruptly ends at the edge without wrapping. Wrapping would make it look more polished and easier to read, especially for longer lines of text.

some components like quote, table are not wrapped properly and just get cut off at the edge of the terminal. Wrapping would ensure that all content is visible and the UI looks more polished. It would also improve readability, especially for users with smaller terminal windows.

---

audit vtcode-\* crates and check if can merged combines any similiar to reduce reduntdant. don;t need check tests

===

vtcode-llm Extraction

The LLM module (50K lines, 132 files) was deferred because it has deep cross-dependencies on vtcode-core internals (utils::http_client, prompts::system, models_manager, tools::traits). A full extraction would require either moving those utilities to separate crates or creating trait abstractions. This is a larger refactoring effort that can be tackled in a future session.
