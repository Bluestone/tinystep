//! Structs representing the JSON Response types from the smallstep API Server.

use serde::Deserialize;

/// The JSON Response from calling:
/// `https://api.smallstep.com/v1/teams/{team name}/authorities/{authority name}`.
#[derive(Clone, Debug, Deserialize)]
pub struct HostedAuthorityResponse {
	/// The fingerprint of the root certificate authority.
	pub fingerprint: String,
	/// The URL to reach this authority at.
	pub url: String,
}

/// The JSON Response from calling:
/// `${smallstep_ca_url}/root/{certificate_fingerprint}`
#[derive(Clone, Debug, Deserialize)]
pub struct StepRootResponse {
	/// The PEM Encoded Certificate authority file.
	pub ca: String,
}

/// The JSON Response from calling:
/// `${smallstep_ca_url}/version`
#[derive(Clone, Debug, Deserialize)]
pub struct StepVersionResponse {
	/// If this server requires client authentication.
	///
	/// We don't actually check this right now, and always assume it's true.
	/// TODO(xxx): can this ever not be true, besides testing?
	#[serde(rename = "requireClientAuthentication")]
	pub require_client_authentication: bool,
	/// The version the server is running.
	pub version: String,
}

/// The JSON response from calling:
/// `${smallstep_ca_url}/health`
#[derive(Clone, Debug, Deserialize)]
pub struct StepHealthResponse {
	/// The status of this smallstep instance.
	///
	/// Currently this is always "ok".
	pub status: String,
}
