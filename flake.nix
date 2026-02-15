{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      forAllSystems =
        fn:
        let
          systems = [
            "x86_64-linux"
            "aarch64-darwin"
          ];
          overlays = [ (import rust-overlay) ];
        in
        nixpkgs.lib.genAttrs systems (
          system:
          fn (
            import nixpkgs {
              inherit system overlays;
            }
          )
        );
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          buildInputs = [
            pkgs.fzf
            pkgs.pnpm
            pkgs.just
            pkgs.cmake
            pkgs.bacon
            pkgs.nodejs
            pkgs.openssl
            pkgs.cargo-dist
            pkgs.pkg-config
            pkgs.rust-analyzer
            pkgs.rust-bin.stable.latest.default
          ];
        };
      });

      packages = forAllSystems (
        pkgs:
        let
          rustPlatform = pkgs.makeRustPlatform {
            cargo = pkgs.rust-bin.stable.latest.default;
            rustc = pkgs.rust-bin.stable.latest.default;
          };

          pname = "penny";
          version = "0.0.16";
        in
        rec {
          ui = pkgs.stdenvNoCC.mkDerivation {
            pname = "${pname}-ui";
            version = version;

            src = ./ui;

            nativeBuildInputs = [
              pkgs.nodejs
              pkgs.pnpmConfigHook
              pkgs.pnpm
            ];

            pnpmDeps = pkgs.fetchPnpmDeps {
              inherit pname version;
              src = ./ui;
              pnpm = pkgs.pnpm;
              fetcherVersion = 3;
              hash =
                if pkgs.stdenv.isLinux then
                  "sha256-iFhK9U811SA6l64Sb98h8Hq9stCw3picqvLwvrLx3Ks="
                else
                  "sha256-lE3HeKgyWUmD4IXtj4RxwjFfFv371bjl+0U2s/Zj3SE=";
            };

            buildPhase = ''
              runHook preBuild

              pnpm build

              runHook postBuild
            '';

            installPhase = ''
              runHook preInstall

              cp -r dist $out
              echo -n "v${version}" > $out/VERSION

              runHook postInstall
            '';
          };

          default = rustPlatform.buildRustPackage {
            inherit pname version;
            src = self;
            buildInputs = [ pkgs.openssl ];
            nativeBuildInputs = [
              pkgs.pkg-config
              pkgs.perl
              pkgs.cmake
            ];
            cargoLock.lockFile = ./Cargo.lock;

            preBuild = ''
              cp -pr --reflink=auto -- ${ui} ui/dist
            '';
          };

          image = pkgs.dockerTools.buildLayeredImage {
            name = pname;
            tag = "latest";
            created = "now";
            contents = [ default ];
            config.ENTRYPOINT = [ "/bin/penny" ];
          };

          deploy = pkgs.writeShellScriptBin "deploy" ''
            ${pkgs.skopeo}/bin/skopeo --insecure-policy copy docker-archive:${image} docker://docker.io/frectonz/penny:${version} --dest-creds="frectonz:$ACCESS_TOKEN"
            ${pkgs.skopeo}/bin/skopeo --insecure-policy copy docker://docker.io/frectonz/penny:${version} docker://docker.io/frectonz/penny:latest --dest-creds="frectonz:$ACCESS_TOKEN"
          '';
        }
      );

      formatter = forAllSystems (
        pkgs:
        pkgs.treefmt.withConfig {
          runtimeInputs = [ pkgs.nixfmt ];
          settings = {
            on-unmatched = "info";
            formatter.nixfmt = {
              command = "nixfmt";
              includes = [ "*.nix" ];
            };
          };
        }
      );
    };
}
