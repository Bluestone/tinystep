//! All of the types for a X509 Cert Bundle Provisioner, these are split out
//! because they're pretty large types so we split it to it's own module for
//! readability sake.

use crate::types::StepProvisionerType;
use serde::Deserialize;

/// Provision certificates using X.509 Cert Bundle to provide authentication
/// so we know which certs to issue.
/// <https://smallstep.com/docs/step-ca/configuration#x5c-x509-certificate>
#[derive(Clone, Debug, Deserialize)]
pub struct StepX5CProvisioner {
	/// The type of this provisioner, will always be:
	/// `StepProvisionerType::X509CertBundle`.
	#[serde(rename = "type")]
	pub typ: StepProvisionerType,
	/// The name given to this provisioner to uniquely identify it.
	pub name: String,
	/// A base64 encoded list of root certificates used for validating X5C
	/// tokens.
	pub roots: String,
	/// An override of "Claims" for this provisioner. This will allow the
	/// provisioner to manually specify the default/min/max tls certificate
	/// issue time if specified.
	#[serde(default)]
	pub claims: Option<super::StepProvisionerClaims>,
}
