{
  rustPlatform,
  pkg-config,
  nix_2_31,
  boost,
  fetchFromGitHub,
  ...
}:
rustPlatform.buildRustPackage {
  name = "buildtime-secrets-nix";

  src = fetchFromGitHub {
    owner = "ttrssreal";
    repo = "buildtime-secrets-nix";
    rev = "87c6fc519cbb954949a3ae87d0c3ad3b925ea22d";
    hash = "sha256-LogILnrCVHFDmeAYLYW2CNdhZ6nVY2vaLI6Tyxwgp8A=";
  };

  cargoHash = "sha256-O3M65SoRWypi+20IXw83nzmsS0gbNtbblQrztsXuYJc=";

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
