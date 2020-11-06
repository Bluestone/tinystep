//! All of the configuration for Smallstep Provisioners, these are kept in
//! their own module, since they are generally very large objects which can
//! make it hard to read if not split out.

use chrono::Duration;
use serde::Deserialize;
use serde_json::Value as JsonValue;

pub mod acme;
pub mod aws;
pub mod azure;
pub mod gcp;
pub mod jwk;
pub mod k8ssa;
pub mod oidc;
pub mod sshpop;
pub mod x5c;

pub use acme::*;
pub use aws::*;
pub use azure::*;
pub use gcp::*;
pub use jwk::*;
pub use k8ssa::*;
pub use oidc::*;
pub use sshpop::*;
pub use x5c::*;

/// Represents all of the provisioner types for a smallstep instance.
/// This is effectively an enum that wraps all of the possible values of
/// the `type` field from a Provisioner Configuration.
#[derive(Clone, Debug, Deserialize)]
pub enum StepProvisionerType {
	/// A Provisioner using a JWK for identities.
	///
	/// <https://smallstep.com/docs/step-ca/configuration#jwk>
	#[serde(rename = "JWK")]
	JsonWebKey,
	/// A Provisioner using OIDC for identities.
	///
	/// <https://smallstep.com/docs/step-ca/configuration#oauthoidc-single-sign-on>
	#[serde(rename = "OIDC")]
	OpenIDConnect,
	/// A Provisioner using an existing GCP instance identity for identity.
	///
	/// <https://smallstep.com/docs/step-ca/configuration#cloud-provisioners>
	#[serde(rename = "GCP")]
	GoogleCloudPlatform,
	/// A Provisioner using an existing AWS instance identity for identity.
	///
	/// <https://smallstep.com/docs/step-ca/configuration#cloud-provisioners>
	#[serde(rename = "AWS")]
	AmazonWebServices,
	/// A Provisioner using an existing Azure instance identity for identity.
	///
	/// <https://smallstep.com/docs/step-ca/configuration#cloud-provisioners>
	#[serde(rename = "Azure")]
	Azure,
	/// A Provisioner using ACME protocol, which can use the normal ACME way of
	/// validating issuing a certificate (e.g. dns/http/etc.)
	///
	/// <https://smallstep.com/docs/step-ca/configuration#acme>
	#[serde(rename = "ACME")]
	Acme,
	/// A Provisioner using an existing X.509 Cert Bundle for identities.
	///
	/// <https://smallstep.com/docs/step-ca/configuration#x5c-x509-certificate>
	#[serde(rename = "X5C")]
	X509CertBundle,
	/// A provisioner using a Kubernetes Service Account for identity.
	///
	/// <https://smallstep.com/docs/step-ca/configuration#k8ssa-kubernetes-service-account>
	#[serde(rename = "K8sSA")]
	KubernetesServiceAccount,
	/// A Provisioner using SSH keys for identity also referred to as "SSHPOP".
	///
	/// <https://smallstep.com/docs/step-ca/configuration#sshpop-ssh-certificate>
	#[serde(rename = "SSHPOP")]
	SshKeypair,
}

impl std::str::FromStr for StepProvisionerType {
	type Err = color_eyre::Report;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"JWK" => Ok(StepProvisionerType::JsonWebKey),
			"OIDC" => Ok(StepProvisionerType::OpenIDConnect),
			"GCP" => Ok(StepProvisionerType::GoogleCloudPlatform),
			"AWS" => Ok(StepProvisionerType::AmazonWebServices),
			"Azure" => Ok(StepProvisionerType::Azure),
			"ACME" => Ok(StepProvisionerType::Acme),
			"X5C" => Ok(StepProvisionerType::X509CertBundle),
			"K8sSA" => Ok(StepProvisionerType::KubernetesServiceAccount),
			"SSHPOP" => Ok(StepProvisionerType::SshKeypair),
			_ => Err(color_eyre::eyre::eyre!(
				"Failed to find provisioner type: {:?}",
				s
			)),
		}
	}
}

/// Represents the "claims" part of a provisioner, which contains generic
/// claims for the actual certificates/keys issued by this provisioner.
/// These are things like min/max/default durations.
#[derive(Clone, Debug, Deserialize)]
pub struct StepProvisionerClaims {
	/// An optional minimum duration for TLS Certificates for this provisioner.
	#[serde(
		rename = "minTLSCertDuration",
		deserialize_with = "crate::types::from_golang_duration_opt",
		default
	)]
	pub min_tls_dur: Option<Duration>,
	/// An optional maximum duration for TLS Certificates for the provisioner.
	#[serde(
		rename = "maxTLSCertDuration",
		deserialize_with = "crate::types::from_golang_duration_opt",
		default
	)]
	pub max_tls_dur: Option<Duration>,
	/// The optional default duration for TLS Certificates for the provisioner.
	#[serde(
		rename = "defaultTLSCertDuration",
		deserialize_with = "crate::types::from_golang_duration_opt",
		default
	)]
	pub default_tls_dur: Option<Duration>,
	/// An optional status of whether or not renewals are disabled.
	///
	/// If not specified assume renewal's aren't disabled.
	#[serde(rename = "disableRenewal", default)]
	pub disable_renewal: Option<bool>,
	/// An optional minimum duration for SSH User Certs issued.
	#[serde(
		rename = "minUserSSHCertDuration",
		deserialize_with = "crate::types::from_golang_duration_opt",
		default
	)]
	pub min_user_ssh_cert_dur: Option<Duration>,
	/// An optional maximum duration for SSH User Certs issued.
	#[serde(
		rename = "maxUserSSHCertDuration",
		deserialize_with = "crate::types::from_golang_duration_opt",
		default
	)]
	pub max_user_ssh_cert_dur: Option<Duration>,
	/// The optional default duration for SSH User Certs issued.
	#[serde(
		rename = "defaultUserSSHCertDuration",
		deserialize_with = "crate::types::from_golang_duration_opt",
		default
	)]
	pub default_user_ssh_cert_duration: Option<Duration>,
	/// An optional minimum duration for SSH Host Certs issued.
	#[serde(
		rename = "minHostSSHCertDuration",
		deserialize_with = "crate::types::from_golang_duration_opt",
		default
	)]
	pub min_host_ssh_cert_duration: Option<Duration>,
	/// An optional maximum duration for SSH Host Certs issued.
	#[serde(
		rename = "maxHostSSHCertDuration",
		deserialize_with = "crate::types::from_golang_duration_opt",
		default
	)]
	pub max_host_ssh_cert_duration: Option<Duration>,
	/// The optional default duration for SSH Host Certs issued.
	#[serde(
		rename = "defaultHostSSHCertDuration",
		deserialize_with = "crate::types::from_golang_duration_opt",
		default
	)]
	pub default_host_ssh_cert_duration: Option<Duration>,
	/// An option that determines if SSH CA has been abled.
	///
	/// If not specified assume it does not exist.
	#[serde(rename = "enableSSHCA", default)]
	pub enable_ssh_ca: Option<bool>,
}

/// The provisioner field `options` is effectively a pair of key/value. This
/// represents the value part of that key/value pair.
#[derive(Clone, Debug, Deserialize)]
pub struct StepProvisionerInnerOptions {
	/// An optional template string.
	#[serde(default)]
	pub template: Option<String>,
	/// An optional template file.
	#[serde(rename = "templateFile", default)]
	pub template_file: Option<String>,
	/// Optional values to render in the template.
	#[serde(rename = "templateData", default)]
	pub template_data: Option<JsonValue>,
}

/// Represents a set of options for a parictular provisioner.
#[derive(Clone, Debug, Deserialize)]
pub struct StepProvisionerOptions {
	/// The SSH Options for this provisioner.
	#[serde(default)]
	pub ssh: Option<StepProvisionerInnerOptions>,
	/// The X509 Options for this provisioner.
	#[serde(default)]
	pub x509: Option<StepProvisionerInnerOptions>,
}

/// Represents an actual provisioner from options, this can be deserailized
/// with a: `deserialize_with` attribute.
#[allow(clippy::pub_enum_variant_names)]
#[derive(Clone, Debug)]
pub enum StepProvisioner {
	/// An OIDC Provisioner.
	OpenIDConnectProvisioner(StepOIDCProvisioner),
	/// A JWK based provisioner.
	JsonWebKeyProvisioner(StepJWKProvisioner),
	/// A GCP based provisioner.
	GoogleCloudPlatformProvisioner(StepGCPProvisioner),
	/// An AWS based provisioner.
	AmazonWebServicesProvisioner(StepAWSProvisioner),
	/// An Azure based provisioner.
	AzureProvisioner(StepAzureProvisioner),
	/// An ACME based provisioner.
	AcmeProvisioner(StepAcmeProvisioner),
	/// A X509 Cert Bundle based provisioner.
	X509CertBundleProvisioner(StepX5CProvisioner),
	/// A Kubernetes Service Account based provisioner.
	KubernetesServiceAccountProvisioner(StepK8SSAProvisioner),
	/// A SSH Certificate based provisioner.
	SshKeypairProvisioner(StepSSHPOPProvisioner),
}
