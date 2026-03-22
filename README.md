# chromium-pwa-wmclass-sync

Automatically fix `StartupWMClass` mismatches and create human-readable companion `.desktop` files for Chromium PWAs. This resolves issues where PWAs are incorrectly grouped under the main browser icon instead of their own, especially on GNOME. This is a known upstream issue tracked at [https://issues.chromium.org/issues/41481818](https://issues.chromium.org/issues/41481818).

For each PWA, the tool creates a companion `.desktop` file with the correct `StartupWMClass` and a human-readable name (e.g., `Google Calendar.desktop`), while hiding the original Chrome-generated file. This enables deterministic configuration management (e.g., for pinning apps to a Dock) and survives Chrome regenerating its files.

This tool is implemented in Rust for high reliability and single-binary deployment.

## Installation

### Nix & Home Manager (Recommended)

This project provides a Home Manager module for easy integration and automated background monitoring.

1. **Add the input to your `flake.nix`:**

    ```nix
    {
      inputs = {
        # ... other inputs
        chromium-pwa-wmclass-sync.url = "github:BohdanTkachenko/chromium-pwa-wmclass-sync";
      };
      
      # ...
    }
    ```

2. **Import the module in your Home Manager configuration:**

    ```nix
    {
      # Enable the service
      programs.chromium-pwa-wmclass-sync.service.enable = true;
    }
    ```

    This will install the tool and set up a `systemd` user service and path unit to watch `~/.local/share/applications/` for changes, automatically fixing new or updated PWA shortcuts.

## Usage

### Automated Service

If you enabled the Home Manager service, the tool runs automatically in the background. It watches your applications directory and triggers the fix whenever files change.

### Manual Execution

You can also run the tool manually if needed:

```bash
chromium-pwa-wmclass-sync [OPTIONS]
```

### Options

- `--dry-run`: Print the changes that would be made without modifying files.
- `-v, --verbose`: Enable verbose logging.
- `--apps-dir <DIR>`: The directory to scan for `.desktop` files (defaults to `~/.local/share/applications/`).

## Development

This project is managed with Nix. You can enter a development shell with all necessary tools (Rust, Cargo, Linter) by running:

```bash
nix develop
```

To run tests and linting:

```bash
nix flake check
```
