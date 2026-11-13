{ lib, rustPlatform, pkg-config, python3, makeWrapper, stdenv,
  alsa-lib, sqlite, libxkbcommon, libGL, vulkan-loader, wayland,
  libx11, libxcursor, libxi, libxrandr }:

let
  root = ../..;
  manifest = builtins.fromTOML (builtins.readFile (root + /Cargo.toml));
  runtimeLibraries = [
    alsa-lib libxkbcommon libGL vulkan-loader wayland
    libx11 libxcursor libxi libxrandr
  ];
in rustPlatform.buildRustPackage {
  pname = "abcom";
  inherit (manifest.package) version;

  src = lib.fileset.toSource {
    inherit root;
    fileset = lib.fileset.unions [
      (root + /Cargo.toml) (root + /Cargo.lock) (root + /build.rs)
      (lib.fileset.fileFilter (file: file.hasExt "rs") (root + /src))
      (root + /assets) (root + /examples)
      (root + /scripts/package_macos.py) (root + /LICENSE)
    ];
  };
  cargoLock.lockFile = root + /Cargo.lock;
  nativeBuildInputs = [ pkg-config ]
    ++ lib.optionals stdenv.hostPlatform.isLinux [ makeWrapper ]
    ++ lib.optionals stdenv.hostPlatform.isDarwin [ python3 ];
  buildInputs = [ sqlite ] ++ lib.optionals stdenv.hostPlatform.isLinux [ alsa-lib ];

  # Network and real-window tests run separately; the build sandbox has neither.
  doCheck = false;
  postInstall = lib.optionalString stdenv.hostPlatform.isLinux ''
    install -Dm644 assets/app_icon.png "$out/share/icons/hicolor/512x512/apps/abcom.png"
    mkdir -p "$out/share/applications"
    cat > "$out/share/applications/abcom.desktop" <<EOF
    [Desktop Entry]
    Type=Application
    Name=Abcom
    Comment=Encrypted peer-to-peer LAN messenger
    Exec=$out/bin/abcom
    Icon=abcom
    StartupWMClass=abcom
    Categories=Network;InstantMessaging;
    Terminal=false
    EOF
    wrapProgram "$out/bin/abcom" \
      --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath runtimeLibraries}"
  '';
  postFixup = lib.optionalString stdenv.hostPlatform.isDarwin ''
    python3 scripts/package_macos.py "$out/bin/abcom" "$out/Applications/Abcom.app"
  '';

  meta = {
    description = "Encrypted peer-to-peer LAN messenger";
    homepage = "https://github.com/abnd-corp/Abcom";
    license = lib.licenses.agpl3Plus;
    platforms = lib.platforms.linux ++ lib.platforms.darwin;
    mainProgram = "abcom";
  };
}
