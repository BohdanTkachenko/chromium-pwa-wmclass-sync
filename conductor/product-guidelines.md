# Product Guidelines

## User Experience (UX) Philosophy
- **Invisible Automation:** The primary goal is "set it and forget it." The tool should operate silently and reliably in the background. Users should ideally never have to interact with it after the initial configuration, with the system state simply being "correct" automatically.

## Engineering Standards
- **Simplicity First:** The codebase should remain lightweight and straightforward. Logic should be contained within a single script or minimal set of files to reduce cognitive load and maintenance overhead. Complexity should be introduced only when absolutely necessary.
- **Minimal Dependencies:** Use lightweight, well-established Rust crates. Leverage Nix for build and dependency management to ensure reliable reproducible builds and long-term stability.

## Documentation & Tone
- **Technical & Precise:** Documentation should be written for a technical audience (developers, sysadmins, NixOS users). It should use precise terminology (e.g., "XDG specification," "systemd unit," "wmclass") and assume a baseline level of system knowledge. The focus is on accuracy and efficiency.
