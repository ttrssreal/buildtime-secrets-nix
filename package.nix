{
  rustPlatform,
  pkg-config,
  nix_2_31,
  boost,
  ...
}:
rustPlatform.buildRustPackage {
  name = "buildtime-secrets-nix";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  doCheck = false;
  strictDeps = true;

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    nix_2_31.dev
    boost.dev
  ];

  meta = {
    description = "Pre-build hook enabling secure, reproducible secret access in derivations";
    longDescription = ''
      A pre-build hook that lets Nix derivations securely and reproducibly express
      dependencies on secrets at build time.
    '';
    mainProgram = "buildtime-secrets-nix";
  };
}
