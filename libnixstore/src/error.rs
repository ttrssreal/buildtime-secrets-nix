use crate::ffi::NixErrorTag;
use std::str::FromStr;
use tracing::warn;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    GenericNixError(String),
    GetVersion(String),
    StorePath(String),
    EnvKeyDoesNotExist(String),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Error::GenericNixError(msg) => msg.clone(),
                Error::GetVersion(msg) => format!("failed to get nix version from store: {msg}"),
                Error::StorePath(msg) => format!("store path not valid: {msg}"),
                Error::EnvKeyDoesNotExist(msg) => format!("while reading derivation: {msg}"),
            }
        )
    }
}

struct TaggedError {
    tag: NixErrorTag,
    msg: String,
}

fn try_parse_tagged_error(
    [tag, msg]: [&str; 2],
) -> std::result::Result<TaggedError, <u8 as FromStr>::Err> {
    Ok(TaggedError {
        tag: NixErrorTag { repr: tag.parse()? },
        msg: msg.to_string(),
    })
}

impl From<cxx::Exception> for Error {
    fn from(value: cxx::Exception) -> Self {
        let destructured: std::result::Result<[&str; 2], _> = value
            .what()
            .splitn(2, ',')
            .collect::<Vec<&str>>()
            .try_into();

        match destructured.map(try_parse_tagged_error) {
            Ok(Ok(TaggedError { tag, msg })) => match tag {
                NixErrorTag::GetVersion => Error::GetVersion(msg),
                NixErrorTag::StorePath => Error::StorePath(msg),
                NixErrorTag::EnvKeyDoesNotExist => Error::EnvKeyDoesNotExist(msg),
                _ => {
                    warn!(
                        "c++ returned an ffi error with an unknown tag \"{}\"",
                        tag.repr
                    );
                    Error::GenericNixError(msg)
                }
            },
            _ => Error::GenericNixError(value.what().to_string()),
        }
    }
}
