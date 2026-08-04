{
  description = "Build a cargo workspace";

  nixConfig = {
    extra-substituters = [
      "https://nix-binary-cache.ken.com.im/nju-cli"
    ];
    extra-trusted-public-keys = [
      "nju-cli:DRWFBO6JKN1QLv2w+o/BgW42BDnBDzebWTP+cwQh71w="
      "nju-cli-cache-1:qG9SW6IO+FJgaSAZraau16eX5aKE+umrhI9oV+K1aHM="
      "ken.com.im:br/oG6ywHr+tGvmUpZEA5mVYSNZgrNrFflazAEI+AK4="
    ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      advisory-db,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        inherit (pkgs) lib;

        craneLib = (crane.mkLib pkgs).overrideToolchain (
          p:
          p.rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" ];
          }
        );
        muslCraneLib = (crane.mkLib pkgs).overrideToolchain (
          p:
          p.rust-bin.stable.latest.default.override {
            targets = [ muslTarget ];
          }
        );
        src = craneLib.cleanCargoSource ./.;

        # Common arguments can be set here to avoid repeating them later
        commonArgs = {
          inherit src;
          strictDeps = true;
          CARGO_BUILD_RUSTFLAGS = lib.optionalString pkgs.stdenv.isDarwin "-C link-arg=-Wl,-dead_strip_dylibs";

          buildInputs = [
            # Add additional build inputs here
          ];

          nativeBuildInputs = [
            pkgs.pkg-config
            # vendored OpenSSL 的 Configure 脚本需要 perl。
            pkgs.perl
          ];
        };

        # Build *just* the cargo dependencies (of the entire workspace),
        # so we can reuse all of that work (e.g. via cachix) when running in CI
        # It is *highly* recommended to use something like cargo-hakari to avoid
        # cache misses when building individual top-level-crates
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        individualCrateArgs = commonArgs // {
          inherit cargoArtifacts;
          inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
          # NB: we disable tests since we'll run them all via cargo-nextest
          doCheck = false;
        };

        nju-cli = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "nju-cli";
            cargoExtraArgs = "-p cli";
          }
        );

        muslTarget = {
          x86_64-linux = "x86_64-unknown-linux-musl";
          aarch64-linux = "aarch64-unknown-linux-musl";
        }.${system};
        muslTargetEnv = lib.toUpper (builtins.replaceStrings [ "-" ] [ "_" ] muslTarget);
        muslTargetEnvLower = builtins.replaceStrings [ "-" ] [ "_" ] muslTarget;
        muslCc = pkgs.pkgsStatic.stdenv.cc;
        muslTargetPrefix = muslCc.targetPrefix;
        muslLinker = "${muslCc}/bin/${muslTargetPrefix}gcc";
        muslBinutils = muslCc.bintools.bintools;
        muslAr = "${muslBinutils}/bin/${muslTargetPrefix}ar";
        muslRanlib = "${muslBinutils}/bin/${muslTargetPrefix}ranlib";

        muslCommonArgs = commonArgs // {
          CARGO_BUILD_TARGET = muslTarget;
          CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
          "CARGO_TARGET_${muslTargetEnv}_LINKER" = muslLinker;

          # vendored OpenSSL 也会编 C 代码，必须使用 musl toolchain。
          "CC_${muslTargetEnvLower}" = muslLinker;
          "CC_${muslTarget}" = muslLinker;
          "AR_${muslTargetEnvLower}" = muslAr;
          "AR_${muslTarget}" = muslAr;
          "RANLIB_${muslTargetEnvLower}" = muslRanlib;
          "RANLIB_${muslTarget}" = muslRanlib;

          nativeBuildInputs = commonArgs.nativeBuildInputs ++ [
            muslCc
          ];
        };

        muslCargoArtifacts = muslCraneLib.buildDepsOnly muslCommonArgs;

        nju-cli-static = muslCraneLib.buildPackage (
          muslCommonArgs
          // {
            pname = "nju-cli-static";
            cargoArtifacts = muslCargoArtifacts;
            inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
            cargoExtraArgs = "-p cli";
            doCheck = false;
          }
        );

        nju-cli-mcp = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "nju-cli-mcp";
            cargoExtraArgs = "-p nju-cli-mcp";
            NJU_CLI_BIN = "${nju-cli}/bin/nju-cli";
            doCheck = false;
          }
        );
      in
      {
        checks = {
          # Build the crates as part of `nix flake check` for convenience
          inherit nju-cli;

          # Run clippy (and deny all warnings) on the workspace source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          my-workspace-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          my-workspace-doc = craneLib.cargoDoc (
            commonArgs
            // {
              inherit cargoArtifacts;
              # This can be commented out or tweaked as necessary, e.g. set to
              # `--deny rustdoc::broken-intra-doc-links` to only enforce that lint
              env.RUSTDOCFLAGS = "--deny warnings";
            }
          );

          # Check formatting
          my-workspace-fmt = craneLib.cargoFmt {
            inherit src;
          };

          my-workspace-toml-fmt = craneLib.taploFmt {
            src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
            # taplo arguments can be further customized below as needed
            # taploExtraArgs = "--config ./taplo.toml";
          };

          # Audit dependencies
          my-workspace-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };

          # Audit licenses
          my-workspace-deny = craneLib.cargoDeny {
            inherit src;
          };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on other crate derivations
          # if you do not want the tests to run twice
          my-workspace-nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
              cargoNextestPartitionsExtraArgs = "--no-tests=pass";
            }
          );

          # Ensure that cargo-hakari is up to date
          my-workspace-hakari = craneLib.mkCargoDerivation {
            inherit src;
            pname = "my-workspace-hakari";
            cargoArtifacts = null;
            doInstallCargoArtifacts = false;

            buildPhaseCargoCommand = ''
              cargo hakari generate --diff  # workspace-hack Cargo.toml is up-to-date
              cargo hakari manage-deps --dry-run  # all workspace crates depend on workspace-hack
              cargo hakari verify
            '';

            nativeBuildInputs = [
              pkgs.cargo-hakari
            ];
          };
        };

        packages = {
          inherit nju-cli nju-cli-mcp;
          default = nju-cli;
        }
        // lib.optionalAttrs pkgs.stdenv.isLinux {
          inherit nju-cli-static;
        };

        apps = {
          nju-cli = flake-utils.lib.mkApp {
            drv = nju-cli;
          };
          nju-cli-mcp = flake-utils.lib.mkApp {
            drv = nju-cli-mcp;
          };
          default = self.apps.${system}.nju-cli;
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};

          # Additional dev-shell environment variables can be set directly
          # MY_CUSTOM_DEVELOPMENT_VAR = "something else";

          # Extra inputs can be added here; cargo and rustc are provided by default.
          packages = [
            pkgs.cargo-hakari
          ];
        };
      }
    );
}
