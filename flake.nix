{
  description = "Abcom — native encrypted LAN messenger";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "aarch64-darwin" "x86_64-darwin" "aarch64-linux" "x86_64-linux" ];
      forAll = nixpkgs.lib.genAttrs systems;
    in {
      packages = forAll (system:
        let pkgs = nixpkgs.legacyPackages.${system};
        in {
          abcom = pkgs.callPackage ./packaging/nix/package.nix { };
          default = self.packages.${system}.abcom;
        });
    };
}
