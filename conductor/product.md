# Initial Concept

# Product Definition

## Goals
- **Fix Window Grouping:** Automatically correct `StartupWMClass` mismatches in Chromium PWA `.desktop` files. This specifically resolves the common issue (e.g., on GNOME) where PWAs are incorrectly grouped under the main browser icon instead of their own.
- **Enable Deterministic Configuration:** Create companion `.desktop` files with human-readable names while hiding the Chrome originals. This overcomes the issue of unstable Chromium PWA IDs, allowing users (especially on NixOS) to reliably reference these applications in configuration files (e.g., for pinning apps to a Dock).
- **Automate Management:** Provide a Home Manager module bundled with systemd service and path units to enable automated, background monitoring and fixing of desktop files.

## Target Audience
- **Linux Desktop Users:** Individuals using Chromium-based browsers for PWAs who experience issues with window management and application launchers.
- **Nix Ecosystem Users:** NixOS and Home Manager users seeking a declarative and automated solution to manage PWA desktop integration.

## Key Features
- **Automated Scanning:** Periodically or event-driven scanning of `~/.local/share/applications/` to identify and process relevant Chromium PWA `.desktop` files.
- **Nix & Home Manager Integration:** Packaged as a Nix flake with a dedicated Home Manager module for seamless deployment and service configuration.
- **Companion File Strategy:** Creates human-readable companion `.desktop` files for each PWA and neuters the originals (`NoDisplay=true`), ensuring Chrome can still find its files while the launcher shows the correct entries. Automatically cleans up companions when PWAs are uninstalled.

## Error Handling & Reporting
- **Resilient Operation:** The tool is designed to fail silently on minor, non-critical errors to ensure the background systemd service continues running without interruption.
- **Detailed Diagnostics:** When encountering malformed `.desktop` files or parsing issues, the tool logs detailed error messages to aid in debugging.

## Constraints & Requirements
- **Efficiency:** Designed to run as a lightweight background service with minimal resource consumption.
- **Standards Compliance:** Adheres to the standard XDG desktop entry specifications to ensure compatibility with various desktop environments.
- **Multi-Profile Support:** Correctly handles PWAs installed from different Chromium profiles (e.g., Default, Work), ensuring they are distinguishable and do not conflict.
