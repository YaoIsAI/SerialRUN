# Contributing to SerialRUN

Thanks for taking the time to contribute. Bug reports, protocol fixes, translations,
documentation updates and pull requests are all welcome. Issues and discussions can be
written in English or Chinese.

## Ways to contribute

- **Report a bug** — use the bug report template and include your OS, adapter model,
  SerialRUN version and clear reproduction steps.
- **Request a feature** — use the feature request template and describe the workflow
  you are trying to support, not only the UI you have in mind.
- **Fix or improve code** — open a focused pull request against `master`.
- **Improve docs or translations** — README, `docs/`, and the website copy in `website/`.
- **Share a plugin** — start from `plugins/serialrun-example-plugin/` and open a PR.

## Development setup

Prerequisites:

- A current stable Rust toolchain (`rustup default stable`)
- A C toolchain for your platform (MSVC Build Tools on Windows, Xcode CLT on macOS,
  `build-essential` on Linux)
- Platform packages required by `eframe`/`egui`; see [docs/BUILD.md](docs/BUILD.md)
  and [docs/BUILD_CN.md](docs/BUILD_CN.md)

```bash
git clone https://github.com/YaoIsAI/SerialRUN.git
cd SerialRUN
cargo build          # debug build of the workspace
cargo run -p serialrun-gui
```

Useful commands:

```bash
cargo fmt --all                    # format
cargo test --workspace             # unit and integration tests
cargo clippy --workspace --all-targets
cargo build --release -p serialrun-gui
```

Hardware-dependent behaviour (baud rates, DTR/RTS, flashing, Modbus timing) cannot be
fully covered by CI. If your change touches those paths, describe the device and test
setup in the pull request.

## Project layout

| Path | Contents |
|------|----------|
| `crates/serialrun-core` | Port handling, protocols, checksums, data logging |
| `crates/serialrun-gui` | Desktop UI (egui/eframe), panels in `src/ui/` |
| `crates/serialrun-cli` | CLI and agent mode |
| `crates/serialrun-mcp` | MCP server and tools |
| `crates/serialrun-plugin-api` | Plugin ABI |
| `plugins/` | Example and community plugins |
| `docs/` | User, build, MCP and release documentation |
| `website/` | Static website (deployed separately) |
| `.github/workflows/` | CI and multi-platform release pipeline |

## Pull request checklist

1. Keep the change focused; separate refactors from behaviour changes.
2. Run `cargo fmt --all` and `cargo test --workspace`.
3. Update README/docs when behaviour or configuration changes.
4. Do not commit build output, device dumps, credentials, tokens or private keys.
5. Describe what changed, why, and how it was tested (including hardware, if relevant).
6. CI must pass on Windows, macOS and Linux.

Use clear commit messages in the imperative mood, for example
`fix: apply typed baud rate on connect` or `docs: document Linux dialout setup`.

## Reporting security issues

Please do not open a public issue for a vulnerability. Use the repository's
**Security → Report a vulnerability** flow so a fix can be prepared before disclosure.

## License

By contributing, you agree that your contributions are licensed under the same
[Business Source License 1.1](LICENSE) as the project.
