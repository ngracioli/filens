# Portability and Support

The goal is to run on Linux, macOS, and Windows computers with a compatible interactive terminal. Verify each combination of operating system, architecture, minimum system version, and terminal before announcing support. The table lists proposed targets for the first release.

| System | Initial target | Terminal test |
| --- | --- | --- |
| Linux x86-64 | `x86_64-unknown-linux-gnu` | Use an interactive terminal. Run the binary on the oldest distribution listed as supported. |
| macOS Apple Silicon | `aarch64-apple-darwin` | Use Terminal.app on a macOS version covered by the release. |
| Windows x86-64 | `x86_64-pc-windows-msvc` | Use Windows Terminal with PowerShell. Verify keyboard shortcuts, resizing, and return to the prompt. |

Linux ARM64, Windows ARM64, and Intel macOS are candidates for expansion. Add each target once an environment is available to run the binary and reproduce issues. Rust's target tiers describe toolchain support. Filens compatibility also depends on its libraries and the terminal session. [Rust targets](https://doc.rust-lang.org/rustc/platform-support.html), [macOS targets](https://doc.rust-lang.org/rustc/platform-support/apple-darwin.html).

## Before Announcing Platform Support

1. Run `cargo test --locked` and `cargo build --release --locked` for the target. Run tests in a native or equivalent environment that can execute the built binary.
2. In a terminal on the target system, open `filens .` and a path with spaces and accented characters. Switch focus, resize the window, and quit with `q`. Confirm that screen, cursor, and input settings are restored. Check the error message for invalid arguments and for execution without an interactive terminal.
3. Test the release in a clean environment without Cargo or a compiler. Publish a binary for each target with its version, requirements, and runtime dependencies. On Linux, document the minimum glibc version verified for the binary. Consider musl if compatibility across distributions becomes a documented problem.
4. Verify native paths and permissions. Where the system permits, cover names that are invalid UTF-8, symbolic links, and entries that disappear while browsing. On Windows, test a drive path. Validate UNC network paths if support for them is advertised.

## Rust Edition and Minimum Version

The project uses the Rust 2024 edition, introduced in Rust 1.85. Edition and minimum compiler version are separate decisions. Set `package.rust-version` after the minimum version passes tests with the code and `Cargo.lock` on the advertised targets. In the locked Linux x86-64 dependency graph, Ratatui 0.30.2 declares Rust 1.88, so Rust 1.85 is already too old for that target. This was checked with `cargo metadata --locked --offline --filter-platform x86_64-unknown-linux-gnu`. The minimum Rust version for Filens still needs verification. [Rust 2024 Edition Guide](https://doc.rust-lang.org/edition-guide/rust-2024/index.html), [Cargo rust-version field](https://doc.rust-lang.org/cargo/reference/rust-version.html).

## Troubleshooting

Record the Filens version, binary target, operating system version, terminal emulator, and steps to reproduce. For layout issues, include the terminal dimensions. For path issues, use an anonymized example that retains the relevant characteristics, such as spaces, accented characters, a symbolic link, or a network drive.
