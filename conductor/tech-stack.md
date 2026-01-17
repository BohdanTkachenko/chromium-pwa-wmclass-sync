# Technology Stack

## Core Application
- **Language:** Rust
- **Key Libraries:** `clap`, `rust-ini`, `log`, `env_logger`, `home`

## Quality Assurance
- **Linter:** `clippy` (integrated into Nix flake check)
- **Testing:** `cargo test` (unit tests), Nix Functional Tests (E2E)

## Packaging & Deployment
- **Package Manager:** Nix (Flakes)
- **Configuration Management:** Home Manager
- **Module System:** NixOS Modules / Home Manager Modules

## System Integration
- **Service Manager:** systemd (User units)
- **Trigger Mechanism:** systemd Path Units (for file monitoring)
