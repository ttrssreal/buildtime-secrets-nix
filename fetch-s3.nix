{
  runCommand,
  awscli2,
  lib,
}:

lib.fetchers.withNormalizedHash { } (
  {
    name,
    s3Path,
    s3Endpoint,
    outputHash,
    outputHashAlgo,
    credentialsSecret,
    ...
  }:
  runCommand name
    {
      nativeBuildInputs = [ awscli2 ];
      requiredSystemFeatures = [ "buildtime-secrets" ];
      requiredSecrets = [ (lib.strings.toJSON credentialsSecret) ];

      inherit outputHash outputHashAlgo;
    }
    ''
      export AWS_SHARED_CREDENTIALS_FILE=/secrets/"${credentialsSecret.name}"
      aws s3 cp --endpoint-url="${s3Endpoint}" s3://"${s3Path}" $out
    ''
)
