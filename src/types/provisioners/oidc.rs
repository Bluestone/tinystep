//! All of the types for an OIDC Provisioner, these are split out because
//! they're pretty large types so we split it to it's own module for
//! readability sake.

use crate::types::StepProvisionerType;
use serde::Deserialize;

/// Provision certificates using OIDC to provide authentication so we know
/// which certs to issue, and who issues them.
/// <https://smallstep.com/docs/step-ca/configuration#oauthoidc-single-sign-on>
#[derive(Clone, Debug, Deserialize)]
pub struct StepOIDCProvisioner {
	/// The type of this provisioner, will always be:
	/// `StepProvisionerType::OpenIDConnect`.
	#[serde(rename = "type")]
	pub typ: StepProvisionerType,
	/// The name given to this provisioner to uniquely identify it.
	pub name: String,
	/// The OAuth2 Client ID that this provisioner should use when authenticating
	/// with OIDC.
	///
	/// The Client ID in OAuth2/OIDC, is used to uniquely identify the
	/// application to the user, so they're aware of what application they are
	/// authenticating too.
	#[serde(rename = "clientID")]
	pub client_id: String,
	/// The OAuth2 Client Secret that this provisioner should use when
	/// authenticating with OIDC.
	///
	/// The Client Secret in OAuth2/OIDC is used to actually perform the
	/// authentication. You may read this is supposed to be secret, and normally
	/// it is. However for Smallstep OIDC is sort of like a mobile app which also
	/// store the client secret publically. The client secret for smallstep is
	/// limited in what it can do (unlike other oauth2 apps), and is whitelisted
	/// which domains it can come from. Thus ensuring only authorized people use
	/// it.
	///
	/// If you followed the setup guide for smallstep:
	/// <https://smallstep.com/docs/step-ca/configuration#oauthoidc-single-sign-on>
	/// This has been configured correctly. You can also see:
	/// <https://tools.ietf.org/html/bcp212> for more info.
	#[serde(rename = "clientSecret")]
	pub client_secret: String,
	/// The configuration URL this provisioner will use when fetching the
	/// configuration needed to perform an OAuth2 Authorization.
	#[serde(rename = "configurationEndpoint")]
	pub configuration_endpoint: String,
	/// The OAuth2 Tenant ID used by smallstep. This is only used for Azure AD
	/// where a Tenant ID is required.
	#[serde(rename = "tenantID", default)]
	pub tenant_id: Option<String>,
	/// A potential list of hand configured admins who are able to get
	/// certificates with custom SANs. If a user is not an admin, it will
	/// only be able to get a certificate with its email in it.
	#[serde(default)]
	pub admins: Option<Vec<String>>,
	/// A potential hand configured list of domains that are actually allowed
	/// to authenticate with OIDC. If present, only users with email from one
	/// of the following domains will be allowed to authenticate.
	#[serde(default)]
	pub domains: Option<Vec<String>>,
	/// A potential hand configured list of groups that are actually allowed to
	/// authenticate with OIDC. If present, only users belonging to the groups
	/// in this list will be able to authenticate.
	#[serde(default)]
	pub groups: Option<Vec<String>>,
	/// An optional loopback address for the client to use when authenticating
	/// with OIDC.
	///
	/// This will only be present when the server to authenticate with doesn't
	/// support the normal standard of using a random port for OIDC flows to be
	/// completed. If specified, and authenticating over OIDC you should use
	/// this address.
	///
	/// The format is documented as being: `:port`, or: `host:port`.
	#[serde(rename = "listenAddress", default)]
	pub listen_address: Option<String>,
	/// An override of "Claims" for this provisioner. This will allow the
	/// provisioner to manually specify the default/min/max tls certificate
	/// issue time if specified.
	#[serde(default)]
	pub claims: Option<super::StepProvisionerClaims>,
	/// An extra set of options for this provisioner specifically. These options
	/// are options that should get passed during the certificate creation
	/// flow, and are internal options to that flow.
	#[serde(default)]
	pub options: Option<super::StepProvisionerOptions>,
}
