//! All of the types for a Kubernetes Service Account Provisioner, these are
//! split out because they're pretty large types so we split it to it's own
//! module for readability sake.

use crate::types::StepProvisionerType;
use serde::Deserialize;

/// Provision certificates using Kubernetes Service Account to provide
/// authentication so we know which certs to issue.
/// <https://smallstep.com/docs/step-ca/configuration#k8ssa-kubernetes-service-account>
#[derive(Clone, Debug, Deserialize)]
pub struct StepK8SSAProvisioner {
	/// The type of this provisioner, will always be:
	/// `StepProvisionerType::KubernetesServiceAccount`.
	#[serde(rename = "type")]
	pub typ: StepProvisionerType,
	/// The name given to this provisioner to uniquely identify it.
	pub name: String,
	/// An override of "Claims" for this provisioner. This will allow the
	/// provisioner to manually specify the default/min/max tls certificate
	/// issue time if specified.
	#[serde(default)]
	pub claims: Option<super::StepProvisionerClaims>,
	/// This is techincally ***mandatory*** for now. One day it may become
	/// optional, however this is not yet implemented. When provided is a base64
	/// encoded list of public keys to validate the kubernetes service account.
	#[serde(rename = "publicKeys")]
	pub public_keys: Option<String>,
}
