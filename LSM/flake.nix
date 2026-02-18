{
  description = "LSM (C)";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
  let
    system = "x86_64-linux";
    pkgs = import nixpkgs { inherit system; };
  in
  {
    devShells.${system}.default = pkgs.mkShell {
      packages = with pkgs; [
        gcc
        gnumake
        pkg-config
        zlib
      ];
    };

    packages.${system}.default = pkgs.stdenv.mkDerivation {
      pname = "memtable";
      version = "0.1.0";

      src = ./.;

      nativeBuildInputs = with pkgs; [
        gnumake
        pkg-config
      ];

      buildInputs = with pkgs; [
        zlib
      ];

      buildPhase = ''
        runHook preBuild
        make
        runHook postBuild
      '';

      installPhase = ''
        runHook preInstall
        mkdir -p $out/bin
        install -m755 bin/app $out/bin/app

        runHook postInstall
      '';
    };
  };
}

