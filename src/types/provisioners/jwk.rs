//! All of the types for a JWK Provisioner, these are split out because they're
//! pretty large types so we split it to it's own module for readability sake.

use crate::types::StepProvisionerType;
use serde::Deserialize;

/// The "Raw" serialized JSON Web Key. PLEASE NOTE: these values are raw
/// values of a JWK. JWKs are notoriously full of footguns, and these
/// raw values make it more so. These are included cause they're a response
/// from `SmallStep`, but please make sure you use these carefully.
///
/// <https://tools.ietf.org/html/rfc7517>
#[derive(Clone, Debug, Deserialize)]
pub struct StepJoseRawWebKey {
	/// The use of this JSON Web Key.
	#[serde(rename = "use", default)]
	pub us: Option<String>,
	/// The Key Type of this JSON Web Key.
	#[serde(default)]
	pub kty: Option<String>,
	/// The JWK value of "kid".
	#[serde(default)]
	pub kid: Option<String>,
	/// The Curve this JSON Web Key is using.
	#[serde(default)]
	pub crv: Option<String>,
	/// The algorithim header of this JWK.
	#[serde(default)]
	pub alg: Option<String>,
	/// Raw value of certain key algorithims that can be populated.
	#[serde(default)]
	pub k: Option<String>,
	/// Raw value of certain key algorithims that can be populated.
	#[serde(default)]
	pub x: Option<String>,
	/// Raw value of certain key algorithims that can be populated.
	#[serde(default)]
	pub y: Option<String>,
	/// Raw value of certain key algorithims that can be populated.
	#[serde(default)]
	pub n: Option<String>,
	/// Raw value of certain key algorithims that can be populated.
	#[serde(default)]
	pub e: Option<String>,
	/// Raw value of certain key algorithims that can be populated.
	#[serde(default)]
	pub d: Option<String>,
	/// Raw value of certain key algorithims that can be populated.
	#[serde(default)]
	pub p: Option<String>,
	/// Raw value of certain key algorithims that can be populated.
	#[serde(default)]
	pub q: Option<String>,
	/// Raw value of certain key algorithims that can be populated.
	#[serde(default)]
	pub dp: Option<String>,
	/// Raw value of certain key algorithims that can be populated.
	#[serde(default)]
	pub dq: Option<String>,
	/// Raw value of certain key algorithims that can be populated.
	#[serde(default)]
	pub qi: Option<String>,
	/// An optional certificate chain for the JWK.
	#[serde(default)]
	pub x5c: Option<Vec<String>>,
	/// An optional certificate url for the JWK.
	#[serde(default)]
	pub x5u: Option<String>,
	/// x5t parameters are base64url-encoded SHA thumbprints
	/// See RFC 7517, Section 4.8, <https://tools.ietf.org/html/rfc7517#section-4.8>
	#[serde(default)]
	pub x5t: Option<String>,
	/// x5t parameters are base64url-encoded SHA thumbprints
	/// See RFC 7517, Section 4.8, <https://tools.ietf.org/html/rfc7517#section-4.8>
	#[serde(rename = "x5t#S256", default)]
	pub x5t_sha256: Option<String>,
}

/// Provision certificates using JWKs to provide authentication so we know
/// which certs to issue. <https://smallstep.com/docs/step-ca/configuration#jwk>
#[derive(Clone, Debug, Deserialize)]
pub struct StepJWKProvisioner {
	/// The type of this provisioner, will always be:
	/// `StepProvisionerType::JsonWebKey`.
	#[serde(rename = "type")]
	pub typ: StepProvisionerType,
	/// The name given to this provisioner to uniquely identify it.
	pub name: String,
	/// The [JSON Web Key](https://tools.ietf.org/html/rfc7517) representation
	/// of a public key used to validate a signed token.
	///
	/// Note this format is relatively "raw", and is full of footguns if not used
	/// from a trusted library.
	pub key: StepJoseRawWebKey,
	/// An optional encrypted private key used to sign tokens. Is encrypted
	/// according to the [JSON Web Encryption](https://tools.ietf.org/html/rfc7516)
	/// standard if present.
	#[serde(rename = "encryptedKey", default)]
	pub encrypted_key: Option<String>,
	/// An override of "Claims" for this provisioner. This will allow the
	/// provisioner to manually specify the default/min/max tls certificate
	/// issue time if specified.
	#[serde(default)]
	pub claims: Option<super::StepProvisionerClaims>,
}
