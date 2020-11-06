//! Tinystep is a library for interacting with a Smallstep CA Server.
//!
//! The goal is to allow you the complete range of functionality of the
//! smallstep cli, just from within a nicely packaged library. As nice as
//! the smallstep cli is, it'd be nice to not just have to shell out to it.
//!
//! This library is currently a heavy work in progress, so although there are
//! no examples here, this is temporary, and we plan to add some here as
//! the library fills out.
//!
//! # Pagination
//!
//! There are certain endpoints provided by the smallstep server that are
//! paginated, and return multiple pages. For every paginated endpoint there
//! are two sets of methods for each of these endpoints. To explain this, and
//! show how they differ we will use the list of provisioners for an example.
//!
//! So we want to get a list of provisioners, there are two sets of endpoints
//! provided by tinystep. We'll use the synchronous endpoints for now, so we
//! we have: `api::provisioners_raw`, and `api::provisioners`.
//! `api::provisioners_raw` is what you use when you want to manually fetch
//! a single page, or manually control pagination yourself. In this
//! example you would manually get all the info you need as an example:
//!
//! ```rust
//! # use tinystep::{api, TinystepClient};
//! # let my_client = TinystepClient::new_from_hosted("bluestone", Some("certs".to_owned())).unwrap();
//! let my_page = api::provisioners_raw(None, &my_client).expect("Failed to fetch provisioners.");
//! println!("Do we have two pages of provisioners? {}", !my_page.next_cursor.is_empty());
//! ```
//!
//! However, this is a bit of a pain for people using this library. Manually
//! managing pagination while it can be useful, usually means your client
//! code has to become more convulted in actual usage. For cases where you
//! don't have some need to manually control pagination, you can use the
//! `api::provisioners` method which returns an iterator that makes only
//! the minimum amount of API Calls necessary. Let's take a look:
//!
//! ```rust
//! # use tinystep::{api, TinystepClient};
//! # let my_client = TinystepClient::new_from_hosted("bluestone", Some("certs".to_owned())).unwrap();
//! let mut found_provisioner = None;
//! for resulting_provisioner in api::provisioners(&my_client) {
//!   let provisioner = resulting_provisioner.unwrap(); // we may have failed
//!   match provisioner {
//!     tinystep::types::StepProvisioner::OpenIDConnectProvisioner(provisioner) => {
//!       if provisioner.name == "GSuite" { found_provisioner = Some(provisioner); break; }
//!     }
//!     _ => continue,
//!   }
//! }
//! assert!(found_provisioner.is_some());
//! ```
//!
//! Here if the provisioner named `GSuite` is on page 1, we'll make one API
//! request. If it's one page 2, we'll make two. Regardless of how many
//! provisioners there are. You can also use asynchronous functions (we'll
//! write it a bit differently to show another way of iterating, but
//! the idea is even asynchronously you can use a stream).
//!
//! ```
//! use futures::stream::{self, StreamExt};
//! # use tokio_test::block_on;
//! # use tinystep::{api, TinystepClient};
//! # let my_client = TinystepClient::new_from_hosted("bluestone", Some("certs".to_owned())).unwrap();
//!
//! async fn find_provisioner(
//!   name: String, client: &TinystepClient
//! ) -> Option<tinystep::types::StepOIDCProvisioner> {
//!   let mut stream = api::provisioners_async(client);
//!   loop {
//!     let item = stream.next().await;
//!     if item.is_none() { return None; }
//!     let resulting_item = item.unwrap();
//!     if resulting_item.is_err() { panic!("Failed to fetch a page: {:?}", resulting_item); }
//!     match resulting_item.unwrap() {
//!       tinystep::types::StepProvisioner::OpenIDConnectProvisioner(prov) => {
//!         if prov.name == name { return Some(prov); }
//!       }
//!       _ => continue,
//!     }
//!   }
//! }
//!
//! # assert!(block_on(find_provisioner("GSuite".to_owned(), &my_client)).is_some());
//! ```

use color_eyre::Result;
use isahc::{
	config::{CaCertificate, SslOption},
	prelude::*,
	HttpClient, ResponseExt,
};
use std::path::PathBuf;
use tracing::{debug, instrument};

pub mod api;
pub use isahc as http_lib;
pub mod types;

/// `TinystepClient` is a small wrapper around an HTTP Client providing a secure
/// channel to communicate with a Smallstep Instance. This should fundamentally
/// be thought of as a "properly configured" HTTP Client. It doesn't implement
/// authentication, or anything like that for you. It simply will figure out
/// how to get your requests to the correct smallstep instance.
///
/// As such it implements very similar methods to `isahc`'s `HTTPClient`, with
/// things like `get`/`post`/`put`/`delete`/`send`. Without implementing
/// anything to do with authentication like `jwk`'s which smallstep can use
/// to authenticate.
///
/// You can construct a tinystep client based off of one of three things:
///
///  1. The root certificate authority file of your smallstep instance, and the
///     url to the smallstep instance. `new_from_ca_file`
///
///  2. The fingerprint of the root certificate authority file for your
///     smallstep instance, and the url to the smallstep instance.
///     Useful for when you don't wanna lug around a CA file. `new_from_fingerprint`
///
///  3. If you're using a hosted version of smallstep, provided by Smallstep
///     you can simply give us your team name, and we will construct the
///     client for you. `new_from_hosted`.
///
/// # Notes
///
/// - `TinystepClient` is entirely thread safe, and the connection pool for it
///   is not cheap to create. We recommend creating one, and reusing it
///   throughout the code.
///
/// - `TinystepClient` assumes the certificate authority does not rotate.
///   If your remote version has a remote certificate authority rotation,
///   you will need to create a new tinystep client.
#[derive(Clone, Debug)]
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
		let resp = isahc::send(req)?.json::<types::StepRootResponse>()?;

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
			fd.write_all(resp.ca.as_bytes())?;
			Ok(PathBuf::from(&file_str))
		}
	}

	/// Construct a HTTP Client from the base URL, and the path to the
	/// certificate authority.
	fn http_client_from_ca_path(path: PathBuf) -> Result<HttpClient> {
		Ok(HttpClient::builder()
			.default_headers(&[(
				"user-agent",
				concat!("tinystep/", env!("CARGO_PKG_VERSION")),
			)])
			.ssl_ca_certificate(CaCertificate::file(path))
			.build()?)
	}

	/// Get the version for a smallstep instance.
	fn get_version(base_url: &str, client: &HttpClient) -> Result<types::StepVersionResponse> {
		Ok(client
			.get(format!("{}/version", base_url))?
			.json::<types::StepVersionResponse>()?)
	}

	/// Connect to any smallstep instance.
	///
	/// This is most likely the most common method you will use for connecting to
	/// smallstep instances. Here simply specify the certificate-authority
	/// of the smallstep instance. This will be not only what smallstep serves
	/// https on, but also what all the certificates are signed by.
	///
	/// If transporting around the certificate file is too much of a pain, you
	/// can look at: `new_from_fingerprint`, and if you're running a hosted
	/// version of smallstep, you can avoid it alltogether with:
	/// `new_from_hosted`.
	#[instrument]
	pub fn new_from_ca_file(mut base_url: String, ca_bundle: PathBuf) -> Result<Self> {
		if base_url.ends_with('/') {
			base_url.pop();
		}
		let http_client = Self::http_client_from_ca_path(ca_bundle)?;
		let version = Self::get_version(&base_url, &http_client)?;

		Ok(Self {
			base_url,
			remote_version: version.version,
			underlying_http_client: http_client,
		})
	}

	/// Connect to any smallstep instance with only the CA fingerprint.
	///
	/// This can be useful in environments, where transporting around the
	/// CA you should be trusting proves difficult. If you don't want to
	/// distribute the certificate file to all your nodes who may be using
	/// smallstep. You can instead transport around the fingerprint of the
	/// certificate.
	///
	/// We will fetch the actual certificate authority from smallstep, validating
	/// it against the fingerprint to ensure we're talking to the correct party.
	///
	/// # Examples
	///
	/// ```rust
	/// # use tinystep::TinystepClient;
	/// let my_client = TinystepClient::new_from_fingerprint(
	///		"https://certs.bluestone.ca.smallstep.com".to_owned(),
	///		"6cbbfb8bf28e552bc710af1de6c76ed0defcf184e518b466c4a707a824ac410d",
	///	).unwrap();
	/// ```
	#[instrument]
	pub fn new_from_fingerprint(mut base_url: String, fingerprint: &str) -> Result<Self> {
		if base_url.ends_with('/') {
			base_url.pop();
		}
		let root_cert_path = Self::get_root_certificate_from_fingerprint(&base_url, fingerprint)?;
		let http_client = Self::http_client_from_ca_path(root_cert_path)?;
		let version = Self::get_version(&base_url, &http_client)?;

		Ok(Self {
			base_url,
			remote_version: version.version,
			underlying_http_client: http_client,
		})
	}

	/// Create a client for interacting with a hosted version of smallstep.
	///
	/// If you are paying for smallstep's hosted SSH Authority, or another
	/// hosted service, this method should allow you to construct a client
	/// from only your team name. By default this assumes you're using
	/// the hosted SSH product, since that's all that is offered, but in the
	/// future you'll be able to override this with the `specific_authority`
	/// argument.
	///
	/// The teamname is the same you go to when you login at:
	/// `https://smallstep.com/app/${ this is your team name }`
	///
	/// # Examples
	///
	/// The simplest way to use a hosted version of smallstep is just to give
	/// your team name, and no specific authority, this will default to using
	/// the only hosted offering of the SSH Authority:
	///
	/// ```no_run
	/// # use tinystep::TinystepClient;
	/// let my_client = TinystepClient::new_from_hosted(
	///   "bluestone",
	///    None,
	/// ).unwrap();
	/// ```
	///
	/// If you want you _can_ manually specify an authority, but there's really
	/// no need to as of right now:
	///
	/// ```rust
	/// # use tinystep::TinystepClient;
	/// let my_client = TinystepClient::new_from_hosted("bluestone", Some("certs".to_owned())).unwrap();
	/// ```
	#[instrument]
	pub fn new_from_hosted(team_name: &str, specific_authority: Option<String>) -> Result<Self> {
		let resp = isahc::get(format!(
			"https://api.smallstep.com/v1/teams/{}/authorities/{}",
			team_name,
			specific_authority.unwrap_or_else(|| "ssh".to_owned())
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

	/// Create a specific URL to the smallstep instance.
	///
	/// Useful for when you want to construct your own request from scratch
	/// to specify things like additional headers, but either don't have
	/// a base-url present, or don't want to pass around the url as an extra
	/// copy.
	///
	/// # Examples
	///
	/// ```rust
	/// # use tinystep::TinystepClient;
	/// let my_client = TinystepClient::new_from_hosted(
	///   "bluestone",
	///   Some("certs".to_owned()),
	/// ).unwrap();
	/// let url_to_use = my_client.construct_url("/version");
	/// // Now I can manually build a request with this url.
	/// # assert_eq!(url_to_use, "https://certs.bluestone.ca.smallstep.com/version");
	/// ```
	#[must_use]
	pub fn construct_url(&self, uri_part: &str) -> String {
		format!("{}{}", self.base_url, uri_part)
	}

	/// Send a DELETE request asynchronously to a particular api route.
	///
	/// To customize the request further you can build the request yourself,
	/// and use `send_async`.
	#[instrument]
	pub async fn delete_async<D>(&self, uri_part: &str) -> Result<D>
	where
		D: serde::de::DeserializeOwned,
	{
		Ok(self
			.underlying_http_client
			.delete_async(format!("{}{}", &self.base_url, uri_part))
			.await?
			.json::<D>()?)
	}

	/// Send a DELETE request to a particular api route.
	///
	/// To customize the request further you can build the request yourself,
	/// and use `send`.
	///
	/// For async function equivalent see `delete_async`.
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

	/// Send a GET request asynchronously to a particular api route.
	///
	/// To customize the request further you can build the request yourself,
	/// and use `send_async`.
	#[instrument]
	pub async fn get_async<D>(&self, uri_part: &str) -> Result<D>
	where
		D: serde::de::DeserializeOwned,
	{
		Ok(self
			.underlying_http_client
			.get_async(format!("{}{}", &self.base_url, uri_part))
			.await?
			.json::<D>()?)
	}

	/// Send a GET request to a particular api route.
	///
	/// To customize the request further you can build the request yourself,
	/// and use `send`.
	///
	/// For async function equivalent see `get_async`.
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

	/// Send a POST request asynchronously to a particular api route.
	///
	/// To customize the request further you can build the request yourself,
	/// and use `send_async`.
	#[instrument(skip(body))]
	pub async fn post_async<D>(&self, uri_part: &str, body: impl Into<isahc::Body>) -> Result<D>
	where
		D: serde::de::DeserializeOwned,
	{
		Ok(self
			.underlying_http_client
			.post_async(format!("{}{}", &self.base_url, uri_part), body)
			.await?
			.json::<D>()?)
	}

	/// Send a POST request to a particular api route.
	///
	/// To customize the request further you can build the request yourself,
	/// and use `send`.
	///
	/// For async function equivalent see `post_async`.
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

	/// Send a PUT request asynchronously to a particular api route.
	///
	/// To customize the request further you can build the request yourself,
	/// and use `send_async`.
	#[instrument(skip(body))]
	pub async fn put_async<D>(&self, uri_part: &str, body: impl Into<isahc::Body>) -> Result<D>
	where
		D: serde::de::DeserializeOwned,
	{
		Ok(self
			.underlying_http_client
			.put_async(format!("{}{}", &self.base_url, uri_part), body)
			.await?
			.json::<D>()?)
	}

	/// Send a PUT request to a particular api route.
	///
	/// To customize the request further you can build the request yourself,
	/// and use `send`.
	///
	/// For async function equivalent see `put_async`.
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

	/// Send any request asynchronously.
	///
	/// You should use this when wanting to fully customize the request you're
	/// sending yourself. If you're unsure of the URL to use, you can use:
	/// `construct_url` in order to get the URL for a particular api route.
	#[instrument(skip(req))]
	pub async fn send_async<B: Into<isahc::Body>, D>(
		&self,
		req: isahc::http::Request<B>,
	) -> Result<D>
	where
		D: serde::de::DeserializeOwned,
	{
		Ok(self
			.underlying_http_client
			.send_async(req)
			.await?
			.json::<D>()?)
	}

	/// Send any request.
	///
	/// You should use this when wanting to fully customize the request you're
	/// sending yourself. If you're unsure of the URL to use, you can use:
	/// `construct_url` in order to get the URL for a particular api route.
	///
	/// For an async function equivalent you can use: `send_async`.
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
}
