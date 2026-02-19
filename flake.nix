{
  description = "Template";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
          overlays = [
            self.overlays.default
          ];
        };
      in
      {
        packages = {
          inherit (pkgs) dt-cli;
          default = pkgs.dt-cli;
        };
      }
    )
    // {
      overlays = {
        default =
          final: prev:
          let
            mtime = self.lastModifiedDate;
            date = builtins.substring 2 6 mtime;
            rev = self.rev or (nixpkgs.lib.warn "Git changes are not committed" (self.dirtyRev or "dirty"));
            version = "${date}+${builtins.substring 0 8 rev}";
          in
          {
            dt-cli = final.rustPlatform.buildRustPackage {
              pname = "dt-cli";
              version = "v0.8.0";
              src = ./.;

              cargoLock.lockFile = ./Cargo.lock;
              cargoBuildFlags = [
                "-p"
                "dt-cli"
              ];

              shellHook = ''
                SHELL="$(grep "^$USER:" /etc/passwd | awk -F: '{ print $NF }')"
                [[ $- == *i* ]] && exec "$SHELL"
              '';
            };
          };
      };
      hydraJobs = self.packages;
    };
}
