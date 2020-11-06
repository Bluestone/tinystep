//! All of the types for a GCP Provisioner, these are split out because they're
//! pretty large types so we split it to it's own module for readability sake.

use crate::types::StepProvisionerType;
use chrono::Duration;
use serde::Deserialize;

/// Provision certificates using a GCP Instance Identity for authentication
/// to know which certs can be issued, and which instance is doing them.
/// <https://smallstep.com/docs/step-ca/configuration#cloud-provisioners>
#[derive(Clone, Debug, Deserialize)]
pub struct StepGCPProvisioner {
	/// The type of this provisioner, will always be:
	/// `StepProvisionerType::GoogleCloudPlatform`.
	#[serde(rename = "type")]
	pub typ: StepProvisionerType,
	/// The name given to this provisioner to uniquely identify it.
	pub name: String,
	/// A list of service accounts which are allowed to issue certificates. If
	/// this list is empty, all service accounts are allowed.
	#[serde(rename = "serviceAccounts")]
	pub service_accounts: Vec<String>,
	/// A list of Project IDs which are allowed to issue certificates. If the
	/// list is empty, all project ids will be allowed.
	#[serde(rename = "projectIDs")]
	pub project_ids: Vec<String>,
	/// By default Custom SANs are allowed for instances, if this is set to true
	/// Custom SANs will be disabled, and instances will only be able to issue
	/// certificates for the DNS of the instance.
	///
	/// The documentation for smallstep calls out the instance dns specifically
	/// as: `<instance-name>.c.<project-id>.internal`
	/// and `<instance-name>.<zone>.c.<project-id>.internal`.
	#[serde(rename = "disableCustomSANs")]
	pub disable_custom_san: bool,
	/// By default cloud identities are only allowed to be used once. This is
	/// to help prevent things like the capital one hack where a SSRF
	/// vulnerability can lead to an escalation of privileges. However,
	/// if this option is set to true, an instance can issue as many certificates
	/// as it wants.
	///
	/// Note: in the smallstep api this is actually called:
	/// `disableTrustOnFirstUse`, it has been renamed to more clearly indicate
	/// what this option does.
	#[serde(rename = "disableTrustOnFirstUse")]
	pub disable_first_use_only: bool,
	/// An optional maximum duration of an instance to grant a certificate.
	#[serde(
		rename = "instanceAge",
		deserialize_with = "crate::types::from_golang_duration_opt",
		default
	)]
	pub instance_age: Option<Duration>,
	/// An override of "Claims" for this provisioner. This will allow the
	/// provisioner to manually specify the default/min/max tls certificate
	/// issue time if specified.
	#[serde(default)]
	pub claims: Option<super::StepProvisionerClaims>,
}
