//! All of the types for a SSH Certificate Provisioner, these are
//! split out because they're pretty large types so we split it to it's own
//! module for readability sake.

use crate::types::StepProvisionerType;
use serde::Deserialize;

/// Provision certificates using SSH Certificate to provide
/// authentication so we know which certs to issue.
/// <https://smallstep.com/docs/step-ca/configuration#sshpop-ssh-certificate>
#[derive(Clone, Debug, Deserialize)]
pub struct StepSSHPOPProvisioner {
	/// The type of this provisioner, will always be:
	/// `StepProvisionerType::SshKeypair`.
	#[serde(rename = "type")]
	pub typ: StepProvisionerType,
	/// The name given to this provisioner to uniquely identify it.
	pub name: String,
	/// An override of "Claims" for this provisioner. This will allow the
	/// provisioner to manually specify the default/min/max tls certificate
	/// issue time if specified.
	#[serde(default)]
	pub claims: Option<super::StepProvisionerClaims>,
}
