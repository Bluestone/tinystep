//! All of the types for an Azure Provisioner, these are split out because they're
//! pretty large types so we split it to it's own module for readability sake.

use crate::types::StepProvisionerType;
use serde::Deserialize;

/// Provision certificates using a Azure Instance Identity for authentication
/// to know which certs can be issued, and which instance is doing them.
/// <https://smallstep.com/docs/step-ca/configuration#cloud-provisioners>
#[derive(Clone, Debug, Deserialize)]
pub struct StepAzureProvisioner {
	/// The type of this provisioner, will always be:
	/// `StepProvisionerType::Azure`.
	#[serde(rename = "type")]
	pub typ: StepProvisionerType,
	/// The name given to this provisioner to uniquely identify it.
	pub name: String,
	/// The Azure account tenant id for this provisioner. This id is the
	/// Directory ID available in the Azure Active Directory properties.
	#[serde(rename = "tenantId")]
	pub tenant_id: String,
	/// A whitelist of resource groups that are allowed to issue certificates.
	/// If this list is empty all resource groups are allowed.
	#[serde(rename = "resourceGroups")]
	pub resource_groups: Vec<String>,
	/// An audience for Azure AD, defaults to: <https://management.azure.com/>,
	/// if not specified.
	#[serde(default)]
	pub audience: Option<String>,
	/// By default Custom SANs are allowed for instances, if this is set to true
	/// Custom SANs will be disabled, and instances will only be able to issue
	/// certificates for the DNS of the instance.
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
	/// An override of "Claims" for this provisioner. This will allow the
	/// provisioner to manually specify the default/min/max tls certificate
	/// issue time if specified.
	#[serde(default)]
	pub claims: Option<super::StepProvisionerClaims>,
}
