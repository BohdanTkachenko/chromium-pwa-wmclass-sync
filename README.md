# chromium-pwa-wmclass-sync

Automatically correct `StartupWMClass` mismatches in Chromium PWA `.desktop` files. This tool resolves issues where PWAs are incorrectly grouped under the main browser icon instead of their own, especially on GNOME. This is a known upstream issue tracked at [https://issues.chromium.org/issues/41481818](https://issues.chromium.org/issues/41481818).

Additionally, it can optionally rename PWA `.desktop` files to use a sanitized, predictable version of the application name, enabling deterministic configuration management (e.g., for pinning apps to a Dock).

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

      # (Optional) Enable file renaming (defaults to false)
      # If enabled, .desktop files will be renamed to match the application name.
      # programs.chromium-pwa-wmclass-sync.rename.enable = true;
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

- `--rename`: Rename `.desktop` files to match the application name (disabled by default).
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
