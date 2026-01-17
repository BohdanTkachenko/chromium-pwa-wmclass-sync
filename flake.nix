{
  description = "A flake for a script to fix Chromium PWA desktop files";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forEachSystem = nixpkgs.lib.genAttrs supportedSystems;

      packages = forEachSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "chromium-pwa-wmclass-sync";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
          };
        }
      );

      checks = forEachSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          package = packages.${system}.default;

          functional = pkgs.stdenv.mkDerivation {
            name = "chromium-pwa-wmclass-sync-functional-test";
            src = ./.;
            buildInputs = [
              packages.${system}.default
              pkgs.nushell
            ];
            dontBuild = true;
            installPhase = ''
              mkdir -p $out
              nu tests/functional-test.nu | tee $out/test-log.txt
            '';
          };
        }
      );

      devShells = forEachSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            buildInputs = [
              pkgs.cargo
              pkgs.rustc
              pkgs.rust-analyzer
              pkgs.clippy
              pkgs.rustfmt
            ];
          };
        }
      );
    in
    {
      inherit packages checks devShells;

      homeManagerModules.default =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        let
          cfg = config.programs.chromium-pwa-wmclass-sync;
        in
        {
          options.programs.chromium-pwa-wmclass-sync = {
            service.enable = lib.mkEnableOption "Chromium PWA desktop file fix service";
            rename.enable = lib.mkOption {
              type = lib.types.bool;
              default = false;
              description = "Whether to rename .desktop files to match application names.";
            };
          };

          config = lib.mkIf cfg.service.enable {
            home.packages = [
              self.packages.${pkgs.stdenv.hostPlatform.system}.default
            ];

            systemd.user.services.chromium-pwa-wmclass-sync = {
              Unit = {
                Description = "Fix Chromium PWA desktop files";
              };
              Service = {
                Type = "oneshot";
                ExecStart = "${self.packages.${pkgs.stdenv.hostPlatform.system}.default}/bin/chromium-pwa-wmclass-sync"
                  + lib.optionalString (cfg.rename.enable) " --rename";
              };
              Install = {
                WantedBy = [ "default.target" ];
              };
            };

            systemd.user.paths.chromium-pwa-wmclass-sync = {
              Unit = {
                Description = "Watch for changes in Chromium PWA desktop files";
              };
              Path = {
                PathChanged = "%h/.local/share/applications/";
              };
              Install = {
                WantedBy = [ "paths.target" ];
              };
            };
          };
        };
    };
}
