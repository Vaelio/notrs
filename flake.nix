# flake.nix. Use by typing 'nix develop' in the main directory of the project
{
  inputs = {
    fenix.url = "github:nix-community/fenix";
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, fenix, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        toolchain = fenix.packages.${system}.stable.rust;

        naersk' = naersk.lib.${system}.override {
          cargo = toolchain;
          rustc = toolchain;
        };
        libPath = with pkgs; lib.makeLibraryPath [
          libGL
          libxkbcommon
	  dbus
          wayland
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];

      in
      {
        packages = {
          notrs = naersk'.buildPackage {
            src = ./.;
            pname = "notrs";
            nativeBuildInputs = [ pkgs.autoPatchelfHook pkgs.makeWrapper ];
            buildInputs = [ pkgs.perl pkgs.pkg-config pkgs.libGL pkgs.libGLU pkgs.libxkbcommon pkgs.wayland pkgs.dbus];
            overrideMain = old: {
              preConfigure = ''
                cargo_build_options="$cargo_build_options --bin notrs"
              '';
            };
            postInstall = ''
              wrapProgram "$out/bin/notrs" --prefix LD_LIBRARY_PATH : "${libPath}"
            '';
          };
        };
        defaultPackage = self.packages.${system}.notrs;
        devShell = with pkgs; mkShell {
          buildInputs = [ cargo rustc rustfmt pre-commit rustPackages.clippy pkg-config libGL libGLU libxkbcommon zenity dbus ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${ with pkgs; lib.makeLibraryPath [ wayland libxkbcommon fontconfig libGL libGLU dbus] }";
        };
      }
    );
}
