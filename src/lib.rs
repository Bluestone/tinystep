use color_eyre::Result;
use isahc::{
	config::{CaCertificate, SslOption},
	prelude::*,
	HttpClient, ResponseExt,
};
use std::path::PathBuf;
use tracing::{debug, instrument};

pub mod types;

/// The underlying HTTP Client used for interacting with a Smallstep
/// Certificate Authority.
#[derive(Debug)]
pub struct TinystepClient {
	/// The Base URL for the smallstep client.
	base_url: String,
	/// The version of the remote smallstep version.
	remote_version: String,
	/// The underlying http client used to make network requests to the smallstep
	/// certificate authority.
	underlying_http_client: HttpClient,
}

impl TinystepClient {
	/// Get the root certificate for a particular smallstep instance based off
	/// it's fingerprint. This writes it out to a file since isahc (because of
	/// curl) requires a filepath.
	fn get_root_certificate_from_fingerprint(base_url: &str, fingerprint: &str) -> Result<PathBuf> {
		// This URL is signed by the root certificate we're fetching.
		let req = Request::get(format!("{}/root/{}", base_url, fingerprint))
			.ssl_options(SslOption::DANGER_ACCEPT_INVALID_CERTS)
			.body(())?;
		let resp = isahc::send(req)?.json::<types::HostedSpecificRootResponse>()?;

		let digest = {
			use openssl::{hash::MessageDigest, x509::X509};
			let raw_digest = X509::from_pem(resp.ca.as_bytes())?.digest(MessageDigest::sha256())?;
			hex::encode(raw_digest).to_lowercase()
		};

		debug!(
			"Received Digest: [{}] from base url: [{}], comparing to argument: [{}]",
			digest, base_url, fingerprint
		);
		if digest != fingerprint {
			return Err(color_eyre::eyre::eyre!(format!(
				"Root certificate for: {} does not match fingerprint: {}",
				base_url, fingerprint
			)));
		}

		{
			use std::{fs::OpenOptions, io::prelude::*};
			let file_str = format!("smallstep-ca-{}.pem", fingerprint);
			let mut fd = OpenOptions::new()
				.create(true)
				.truncate(true)
				.write(true)
				.read(false)
				.open(&file_str)?;
			fd.write(resp.ca.as_bytes())?;
			Ok(PathBuf::from(&file_str))
		}
	}

	/// Construct a HTTP Client from the base URL, and the path to the
	/// certificate authority.
	///
	/// `path`: the path to the ca file.
	fn http_client_from_ca_path(path: PathBuf) -> Result<HttpClient> {
		Ok(HttpClient::builder()
			.default_headers(&[(
				"user-agent",
				concat!("tinystep/", env!("CARGO_PKG_VERSION")),
			)])
			.ssl_ca_certificate(CaCertificate::file(path))
			.build()?)
	}

	/// Get the version for a particular base url.
	///
	/// `base_url`: The Base URL to fetch from.
	/// `client`: The HTTP Client to use to fetch the version.
	fn get_version(base_url: &str, client: &HttpClient) -> Result<types::StepVersionResponse> {
		Ok(client
			.get(format!("{}/version", base_url))?
			.json::<types::StepVersionResponse>()?)
	}

	/// Create a tinystep client from a team name, and specific authroity.
	///
	/// `team_name`: the name of the team you use to sign into smallstep.
	/// `specific_authority`: for when hosted smallstep supports multiple
	///                       certificate authorities, right now it supports
	///                       one, ssh.
	#[instrument]
	pub fn new_hosted(team_name: &str, specific_authority: Option<String>) -> Result<Self> {
		let resp = isahc::get(format!(
			"https://api.smallstep.com/v1/teams/{}/authorities/{}",
			team_name,
			specific_authority.unwrap_or("ssh".to_owned())
		))?
		.json::<types::HostedAuthorityResponse>()?;
		let root_cert_path =
			Self::get_root_certificate_from_fingerprint(&resp.url, &resp.fingerprint)?;
		let http_client = Self::http_client_from_ca_path(root_cert_path)?;
		let version = Self::get_version(&resp.url, &http_client)?;

		Ok(Self {
			base_url: resp.url,
			remote_version: version.version,
			underlying_http_client: http_client,
		})
	}

	/// Create a client to connect to a smallstep-ca instance.
	///
	/// `base_url`: the base url for the smallstep-ca server.
	/// `fingerprint`: The fingerprint of the root certificate.
	#[instrument]
	pub fn new_from_fingerprint(base_url: String, fingerprint: &str) -> Result<Self> {
		let root_cert_path = Self::get_root_certificate_from_fingerprint(&base_url, fingerprint)?;
		let http_client = Self::http_client_from_ca_path(root_cert_path)?;
		let version = Self::get_version(&base_url, &http_client)?;

		Ok(Self {
			base_url: base_url,
			remote_version: version.version,
			underlying_http_client: http_client,
		})
	}

	/// Create a client to connect to a smallstep-ca instance.
	///
	/// `base_url`: the base url for the smallstep-ca server.
	/// `ca_bundle`: the file path to the CA Certificate.
	#[instrument]
	pub fn new_from_ca_file(base_url: String, ca_bundle: PathBuf) -> Result<Self> {
		let http_client = Self::http_client_from_ca_path(ca_bundle)?;
		let version = Self::get_version(&base_url, &http_client)?;

		Ok(Self {
			base_url: base_url,
			remote_version: version.version,
			underlying_http_client: http_client,
		})
	}

	/// Construct a URL to the Smallstep http instance.
	///
	/// `uri_part`: the uri part to append to the base url.
	pub fn construct_url(&self, uri_part: &str) -> String {
		format!("{}{}", self.base_url, uri_part)
	}

	#[instrument]
	pub fn delete<D>(&self, uri_part: &str) -> Result<D>
	where
		D: serde::de::DeserializeOwned,
	{
		Ok(self
			.underlying_http_client
			.delete(format!("{}{}", &self.base_url, uri_part))?
			.json::<D>()?)
	}

	#[instrument]
	pub fn get<D>(&self, uri_part: &str) -> Result<D>
	where
		D: serde::de::DeserializeOwned,
	{
		Ok(self
			.underlying_http_client
			.get(format!("{}{}", &self.base_url, uri_part))?
			.json::<D>()?)
	}

	#[instrument(skip(body))]
	pub fn post<D>(&self, uri_part: &str, body: impl Into<isahc::Body>) -> Result<D>
	where
		D: serde::de::DeserializeOwned,
	{
		Ok(self
			.underlying_http_client
			.post(format!("{}{}", &self.base_url, uri_part), body)?
			.json::<D>()?)
	}

	#[instrument(skip(body))]
	pub fn put<D>(&self, uri_part: &str, body: impl Into<isahc::Body>) -> Result<D>
	where
		D: serde::de::DeserializeOwned,
	{
		Ok(self
			.underlying_http_client
			.put(format!("{}{}", &self.base_url, uri_part), body)?
			.json::<D>()?)
	}

	#[instrument(skip(req))]
	pub fn send<B: Into<isahc::Body>, D>(&self, req: isahc::http::Request<B>) -> Result<D>
	where
		D: serde::de::DeserializeOwned,
	{
		Ok(self.underlying_http_client.send(req)?.json::<D>()?)
	}
}

#[cfg(test)]
mod unit_tests {
	use super::*;

	#[test]
	pub fn test_is_send_and_sync() {
		fn is_send<T: Send>() {}
		fn is_sync<T: Sync>() {}

		is_send::<TinystepClient>();
		is_sync::<TinystepClient>();
	}

	#[test]
	pub fn test_from_fingerprint() {
		let res = TinystepClient::new_from_fingerprint(
			"https://ssh.bluestone.ca.smallstep.com".to_owned(),
			"0cc9ff5094903449db5abab8a195df4e803c298e92dc9ce821419b7ad6cc35a4",
		);
		println!("{:?}", res);
		assert!(res.is_ok());
	}

	#[test]
	pub fn test_derive_teamname() {
		let res = TinystepClient::new_hosted("bluestone", None);
		println!("{:?}", res);
		assert!(res.is_ok());

		let other_res = TinystepClient::new_hosted("bluestone", Some("ssh".to_string()));
		println!("{:?}", other_res);
		assert!(res.is_ok());
	}
}
