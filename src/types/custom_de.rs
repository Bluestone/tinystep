//! A series of custom deserializers for the types provided by a smallstep
//! server.

use crate::types::{
	StepAWSProvisioner, StepAcmeProvisioner, StepAzureProvisioner, StepGCPProvisioner,
	StepJWKProvisioner, StepK8SSAProvisioner, StepOIDCProvisioner, StepProvisioner,
	StepProvisionerType, StepSSHPOPProvisioner, StepX5CProvisioner,
};
use chrono::Duration;
use serde::{
	de::{Deserializer, Error as DeError, Unexpected as DeUnexpected},
	Deserialize,
};
use serde_json::Value as JsonValue;
use std::str::FromStr;

/// When performing a custom deserialization, and run into an unexpected type
/// you need to tell serde what type you ran into that was unexpected, this
/// simplifies that for json deserialization by figuring it out for you.
#[must_use]
pub fn find_unknown_type(to_find_type: &JsonValue) -> DeUnexpected {
	if to_find_type.is_array() {
		DeUnexpected::Other("array")
	} else if to_find_type.is_boolean() {
		DeUnexpected::Bool(to_find_type.as_bool().unwrap())
	} else if to_find_type.is_number() {
		if to_find_type.is_i64() {
			DeUnexpected::Signed(to_find_type.as_i64().unwrap())
		} else if to_find_type.is_u64() {
			DeUnexpected::Unsigned(to_find_type.as_u64().unwrap())
		} else if to_find_type.is_f64() {
			DeUnexpected::Float(to_find_type.as_f64().unwrap())
		} else {
			DeUnexpected::Other("Number")
		}
	} else if to_find_type.is_object() {
		DeUnexpected::Map
	} else if to_find_type.is_string() {
		DeUnexpected::Str(to_find_type.as_str().unwrap())
	} else if to_find_type.is_null() {
		DeUnexpected::Unit
	} else {
		DeUnexpected::Other("unknown type")
	}
}

/// Parsing a `time.Duration` from golang. Can be used with the
/// `deserialize_with` attribute for serde.
///
/// This actually isn't perfect and does rounding of second or less float
/// values. This is okay for smallstep, but note: may not be perfect
/// for you.
///
/// # Errors
///
/// * `DeError::invalid_type` - when the type is not a string containing a duration.
/// * `DeError::custom` - Invalid timestamp.
/// * `DeError::custom` - time overflow.
#[allow(clippy::too_many_lines, clippy::cast_possible_truncation)]
pub fn from_golang_duration<'a, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
	D: Deserializer<'a>,
{
	let parsed_str = JsonValue::deserialize(deserializer)?;
	if !parsed_str.is_string() {
		return Err(DeError::invalid_type(
			find_unknown_type(&parsed_str),
			&"A string duration",
		));
	}
	let full_str = parsed_str.to_string();
	let subtract_quotes = full_str.trim_start_matches('"').trim_end_matches('"');
	let should_negate = subtract_quotes.starts_with('-');
	let dur_as_str = subtract_quotes
		.trim_start_matches('-')
		.trim_start_matches('+');

	let mut dur = Duration::zero();
	let mut tmp_number_str = String::new();
	let mut skip_next_char = false;

	for (idx, car) in dur_as_str.chars().enumerate() {
		if skip_next_char {
			skip_next_char = false;
			continue;
		}
		let as_u8 = car as u8;
		if car == '.' || (as_u8 >= (b'0') && as_u8 <= (b'9')) {
			tmp_number_str.push(car);
		} else {
			match car {
				'h' => {
					let pfr = tmp_number_str.parse();
					if pfr.is_err() {
						return Err(DeError::custom(format!(
							"Invalid time duration: `{}`",
							tmp_number_str
						)));
					}
					tmp_number_str = String::new();
					let parsed_float: f64 = pfr.unwrap();

					let potential_new_dur = dur.checked_add(&Duration::seconds(
						((parsed_float * (60_f64)) * 60_f64).round() as i64,
					));
					if potential_new_dur.is_none() {
						return Err(DeError::custom("overflow time!"));
					}
					dur = potential_new_dur.unwrap();
				}
				'm' => {
					if idx + 1 < dur_as_str.len()
						&& dur_as_str.get(idx + 1..idx + 2).unwrap() == "s"
					{
						skip_next_char = true;
						let pfr = tmp_number_str.parse();
						if pfr.is_err() {
							return Err(DeError::custom(format!(
								"Invalid time duration: `{}`",
								tmp_number_str
							)));
						}
						tmp_number_str = String::new();
						let parsed_float: f64 = pfr.unwrap();

						let potential_new_dur =
							dur.checked_add(&Duration::milliseconds(parsed_float.round() as i64));
						if potential_new_dur.is_none() {
							return Err(DeError::custom("overflow time!"));
						}
						dur = potential_new_dur.unwrap();
					} else {
						let pfr = tmp_number_str.parse();
						if pfr.is_err() {
							return Err(DeError::custom(format!(
								"Invalid time duration: `{}`",
								tmp_number_str
							)));
						}
						tmp_number_str = String::new();
						let parsed_float: f64 = pfr.unwrap();

						let potential_new_dur = dur.checked_add(&Duration::seconds(
							(parsed_float * (60_f64)).round() as i64,
						));
						if potential_new_dur.is_none() {
							return Err(DeError::custom("overflow time!"));
						}
						dur = potential_new_dur.unwrap();
					}
				}
				'u' | 'µ' | 'μ' => {
					if idx + 1 < dur_as_str.len()
						&& dur_as_str.get(idx + 1..idx + 2).unwrap() == "s"
					{
						skip_next_char = true;
						let pfr = tmp_number_str.parse();
						if pfr.is_err() {
							return Err(DeError::custom(format!(
								"Invalid time duration: `{}`",
								tmp_number_str
							)));
						}
						tmp_number_str = String::new();
						let parsed_float: f64 = pfr.unwrap();

						let potential_new_dur =
							dur.checked_add(&Duration::microseconds(parsed_float.round() as i64));
						if potential_new_dur.is_none() {
							return Err(DeError::custom("overflow time!"));
						}
						dur = potential_new_dur.unwrap();
					} else {
						return Err(DeError::custom(
							"Unknown time duration, isn't `h`, `m`, `s`, `us`, `ms`, `ns`",
						));
					}
				}
				'n' => {
					if idx + 1 < dur_as_str.len()
						&& dur_as_str.get(idx + 1..idx + 2).unwrap() == "s"
					{
						skip_next_char = true;
						let pfr = tmp_number_str.parse();
						if pfr.is_err() {
							return Err(DeError::custom(format!(
								"Invalid time duration: `{}`",
								tmp_number_str
							)));
						}
						tmp_number_str = String::new();
						let parsed_float: f64 = pfr.unwrap();

						let potential_new_dur =
							dur.checked_add(&Duration::nanoseconds(parsed_float.round() as i64));
						if potential_new_dur.is_none() {
							return Err(DeError::custom("overflow time!"));
						}
						dur = potential_new_dur.unwrap();
					} else {
						return Err(DeError::custom(
							"Unknown time duration, isn't `h`, `m`, `s`, `us`, `ms`, `ns`",
						));
					}
				}
				's' => {
					let pfr = tmp_number_str.parse();
					if pfr.is_err() {
						return Err(DeError::custom(format!(
							"Invalid time duration: `{}`",
							tmp_number_str
						)));
					}
					tmp_number_str = String::new();
					let parsed_float: f64 = pfr.unwrap();

					let potential_new_dur =
						dur.checked_add(&Duration::seconds(parsed_float.round() as i64));
					if potential_new_dur.is_none() {
						return Err(DeError::custom("overflow time!"));
					}
					dur = potential_new_dur.unwrap();
				}
				_ => {
					return Err(DeError::custom(
						"Unknown time duration, isn't `h`, `m`, `s`, `us`, `ms`, `ns`",
					));
				}
			}
		}
	}

	if should_negate {
		Ok(-dur)
	} else {
		Ok(dur)
	}
}

/// Parse out an optional golang duration, this will turn all errors into:
/// `Ok(None)`. This should never actually error itself. Can be used with the
/// `deserialize_with` attribute for serde.
///
/// # Errors
///
/// This method will never error, as it maps errors to: `Ok(None)`.
pub fn from_golang_duration_opt<'a, D>(
	deserializer: D,
) -> std::result::Result<Option<Duration>, D::Error>
where
	D: Deserializer<'a>,
{
	let from_dur_raw = from_golang_duration(deserializer);
	if let Ok(from_dur_unwrapped) = from_dur_raw {
		Ok(Some(from_dur_unwrapped))
	} else {
		Ok(None)
	}
}

/// Deserialize a list of provisioners. This is called
/// `dynamic_provisioner_list` because smallstep identifies provisioners
/// by a "type" field, which is dynamic itself. Can be used with the
/// `deserialize_with` attribute for serde.
///
/// # Errors
///
/// * `DeError::invalid_type` - when not an array of objects.
/// * `DeError::invalid_type` - when the items in the array are not objects.
/// * `DeError::invalid_type` - when there is no type field that is a string.
/// * `DeError::unknown_variant` - unknown provisioner type.
/// * `DeError::custom` - invalid parsed object.
#[allow(clippy::too_many_lines)]
pub fn dynamic_provisioner_list<'a, D>(
	deserializer: D,
) -> std::result::Result<Vec<StepProvisioner>, D::Error>
where
	D: Deserializer<'a>,
{
	let as_any = JsonValue::deserialize(deserializer)?;
	if !as_any.is_array() {
		return Err(DeError::invalid_type(
			find_unknown_type(&as_any),
			&"an array of provisioner objects",
		));
	}

	let mut result = Vec::new();
	for any in as_any.as_array().unwrap() {
		if !any.is_object() {
			return Err(DeError::invalid_type(
				find_unknown_type(&any),
				&"a provisioner object",
			));
		}

		if !any["type"].is_string() {
			return Err(DeError::invalid_type(
				find_unknown_type(&any["type"]),
				&"A string `type` that identifies this provisioner",
			));
		}

		let type_str = any["type"].as_str().unwrap();
		let attempt_enum_match = StepProvisionerType::from_str(type_str);
		if attempt_enum_match.is_err() {
			return Err(DeError::unknown_variant(
				type_str,
				&[
					"JWK", "OIDC", "GCP", "AWS", "Azure", "ACME", "X5C", "K8sSA", "SSHPOP",
				],
			));
		}

		match attempt_enum_match.unwrap() {
			StepProvisionerType::JsonWebKey => {
				let res = serde_json::from_value::<StepJWKProvisioner>(any.clone());
				if let Err(err_case) = res {
					return Err(DeError::custom(err_case.to_string()));
				}
				result.push(StepProvisioner::JsonWebKeyProvisioner(res.unwrap()));
			}
			StepProvisionerType::OpenIDConnect => {
				let res = serde_json::from_value::<StepOIDCProvisioner>(any.clone());
				if let Err(err_case) = res {
					return Err(DeError::custom(err_case.to_string()));
				}
				result.push(StepProvisioner::OpenIDConnectProvisioner(res.unwrap()));
			}
			StepProvisionerType::GoogleCloudPlatform => {
				let res = serde_json::from_value::<StepGCPProvisioner>(any.clone());
				if let Err(err_case) = res {
					return Err(DeError::custom(err_case.to_string()));
				}
				result.push(StepProvisioner::GoogleCloudPlatformProvisioner(
					res.unwrap(),
				));
			}
			StepProvisionerType::AmazonWebServices => {
				let res = serde_json::from_value::<StepAWSProvisioner>(any.clone());
				if let Err(err_case) = res {
					return Err(DeError::custom(err_case.to_string()));
				}
				result.push(StepProvisioner::AmazonWebServicesProvisioner(res.unwrap()));
			}
			StepProvisionerType::Azure => {
				let res = serde_json::from_value::<StepAzureProvisioner>(any.clone());
				if let Err(err_case) = res {
					return Err(DeError::custom(err_case.to_string()));
				}
				result.push(StepProvisioner::AzureProvisioner(res.unwrap()));
			}
			StepProvisionerType::Acme => {
				let res = serde_json::from_value::<StepAcmeProvisioner>(any.clone());
				if let Err(err_case) = res {
					return Err(DeError::custom(err_case.to_string()));
				}
				result.push(StepProvisioner::AcmeProvisioner(res.unwrap()));
			}
			StepProvisionerType::X509CertBundle => {
				let res = serde_json::from_value::<StepX5CProvisioner>(any.clone());
				if let Err(err_case) = res {
					return Err(DeError::custom(err_case.to_string()));
				}
				result.push(StepProvisioner::X509CertBundleProvisioner(res.unwrap()));
			}
			StepProvisionerType::KubernetesServiceAccount => {
				let res = serde_json::from_value::<StepK8SSAProvisioner>(any.clone());
				if let Err(err_case) = res {
					return Err(DeError::custom(err_case.to_string()));
				}
				result.push(StepProvisioner::KubernetesServiceAccountProvisioner(
					res.unwrap(),
				));
			}
			StepProvisionerType::SshKeypair => {
				let res = serde_json::from_value::<StepSSHPOPProvisioner>(any.clone());
				if let Err(err_case) = res {
					return Err(DeError::custom(err_case.to_string()));
				}
				result.push(StepProvisioner::SshKeypairProvisioner(res.unwrap()));
			}
		}
	}

	Ok(result)
}

#[cfg(test)]
mod unit_test {
	use super::*;

	#[derive(Clone, Debug, Deserialize)]
	pub struct DurationOption {
		#[serde(deserialize_with = "from_golang_duration_opt", default)]
		pub field_a: Option<Duration>,
		#[serde(deserialize_with = "from_golang_duration")]
		pub field_b: Duration,
	}

	#[test]
	pub fn test_deserialize() {
		let string_a = r#"
		{
			"field_b": "-1.5h"
		}
		"#;
		let string_b = r#"
		{
			"field_a": "+300ms",
			"field_b": "2h45m"
		}
		"#;

		let parsed_a = serde_json::from_str::<DurationOption>(string_a);
		println!("{:?}", parsed_a);
		let the_a = parsed_a.unwrap();
		assert!(the_a.field_a.is_none());
		assert_eq!(the_a.field_b.num_seconds(), -5400);

		let parsed_b = serde_json::from_str::<DurationOption>(string_b);
		println!("{:?}", parsed_b);
		let the_b = parsed_b.unwrap();
		assert!(the_b.field_a.is_some());
		assert_eq!(the_b.field_a.unwrap().num_milliseconds(), 300);
		assert_eq!(the_b.field_b.num_seconds(), 9900);
	}
}
