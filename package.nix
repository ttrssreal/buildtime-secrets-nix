{
  rustPlatform,
  pkg-config,
  nix_2_31,
  boost,
  ...
}:
rustPlatform.buildRustPackage {
  name = "buildtime-secrets-pre-build-hook";

  src = ./.;

  doCheck = false;
  strictDeps = true;

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    nix_2_31.dev
    boost.dev
  ];

  cargoHash = "sha256-Ug7vUeQVmxevMpCIfPdlvCekLPoZjIvuF1LYNbH9NBc=";

  meta = {
    description = "Pre-build hook enabling secure, reproducible secret access in derivations";
    longDescription = ''
      A pre-build hook that lets Nix derivations securely and reproducibly express
      dependencies on secrets at build time.
    '';
    mainProgram = "buildtime-secrets-nix";
  };
}
