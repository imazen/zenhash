use core::fmt;

/// Errors returned by this crate. Wrapped in [`whereat::At`] for location traces.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// A custom XXH3 secret is shorter than [`XXH3_SECRET_SIZE_MIN`](crate::XXH3_SECRET_SIZE_MIN)
    /// bytes.
    SecretTooShort {
        /// Length of the rejected secret, in bytes.
        len: usize,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SecretTooShort { len } => write!(
                f,
                "XXH3 secret is {len} bytes; at least {} are required",
                crate::XXH3_SECRET_SIZE_MIN
            ),
        }
    }
}

impl core::error::Error for Error {}
