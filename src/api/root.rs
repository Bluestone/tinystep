//! API Calls that begin with: `/root/` in their URL.

use crate::{types::StepRootResponse, TinystepClient};
use color_eyre::Result;
use tracing::instrument;

/// Get the root certificate for a particular fingerprint.
///
/// Useful when you need to turn a certificate fingerprint/sha
/// into the actual certificate.
///
/// For an asynchronous version of this method look at: `for_fingerprint_async`.
#[instrument]
pub fn for_fingerprint(fingerprint: &str, client: &TinystepClient) -> Result<StepRootResponse> {
	client.get(&format!("/root/{}", fingerprint))
}

/// Get the root certificate for a particular fingerprint.
///
/// Useful when you need to turn a certificate fingerprint/sha
/// into the actual certificate.
#[instrument]
pub async fn for_fingerprint_async(
	fingerprint: &str,
	client: &TinystepClient,
) -> Result<StepRootResponse> {
	client.get_async(&format!("/root/{}", fingerprint)).await
}
