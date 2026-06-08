//! `PreviewError` — error types for the `slideforge-preview` crate.
//!
//! All fallible operations in the preview server return `Result<_, PreviewError>`.
//! Errors are rendered by the CLI using `miette` with colored source pointers.

/// Errors produced by the `slideforge-preview` crate.
///
/// # AC-009 / BC-4.03.004 EC-003
///
/// [`PreviewError::PortInUse`] is returned by [`crate::PreviewServer::start`] when
/// the requested port is already bound. The error message includes the hint:
/// `"Use --port <N> to specify a different port."`
#[derive(Debug, thiserror::Error)]
pub enum PreviewError {
    /// The requested port is already in use.
    ///
    /// # Hint
    ///
    /// The error message includes `"Use --port <N> to specify a different port."`.
    #[error("Port {port} is already in use. Use --port <N> to specify a different port.")]
    PortInUse {
        /// The port that was requested but could not be bound.
        port: u16,
    },

    /// An I/O error occurred while binding the TCP listener.
    #[error("I/O error binding preview server: {0}")]
    Io(#[from] std::io::Error),

    /// The OS CSPRNG was unavailable and a CSP nonce could not be generated.
    ///
    /// This is extremely rare and indicates a severely broken OS environment.
    /// Carries the underlying `getrandom` error string.
    #[error("Failed to generate CSP nonce (OS CSPRNG unavailable): {0}")]
    Nonce(String),
}
