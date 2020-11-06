//! A module containing all of the HTTP Responses from a smallstep server.

use crate::{types::StepProvisioner, TinystepClient};
use color_eyre::Result;
use futures::{
	future::FutureExt,
	task::{Context, Poll},
	Stream,
};
use serde::Deserialize;
use std::{future::Future, pin::Pin};

/// The JSON Response from calling:
/// `https://api.smallstep.com/v1/teams/{team name}/authorities/{authority name}`.
#[derive(Clone, Debug, Deserialize)]
pub struct HostedAuthorityResponse {
	/// The fingerprint of the root certificate authority.
	pub fingerprint: String,
	/// The URL to reach this authority at.
	pub url: String,
}

/// The JSON Response from calling:
/// `${smallstep_ca_url}/root/{certificate_fingerprint}`
#[derive(Clone, Debug, Deserialize)]
pub struct StepRootResponse {
	/// The PEM Encoded Certificate authority file.
	pub ca: String,
}

/// The JSON Response from calling:
/// `${smallstep_ca_url}/version`
#[derive(Clone, Debug, Deserialize)]
pub struct StepVersionResponse {
	/// If this server requires client authentication.
	///
	/// We don't actually check this right now, and always assume it's true.
	/// TODO(xxx): can this ever not be true, besides testing?
	#[serde(rename = "requireClientAuthentication")]
	pub require_client_authentication: bool,
	/// The version the server is running.
	pub version: String,
}

/// The JSON response from calling:
/// `${smallstep_ca_url}/health`
#[derive(Clone, Debug, Deserialize)]
pub struct StepHealthResponse {
	/// The status of this smallstep instance.
	///
	/// Currently this is always "ok".
	pub status: String,
}

/// The JSON response from calling:
/// `${smallstep_ca_url}/provisioners`
///
/// The main difference between this, and `StepPartialProvisionersResponse`
/// is this is not iterable, and provides the raw `next_cursor`.
#[derive(Clone, Debug, Deserialize)]
pub struct StepProvisionersResponseRaw {
	/// The list of provisioners.
	#[serde(deserialize_with = "crate::types::dynamic_provisioner_list")]
	pub provisioners: Vec<StepProvisioner>,
	/// A cursor for a next page, rather than being optional this is an empty
	/// string if there is no next page.
	#[serde(rename = "nextCursor")]
	pub next_cursor: String,
}

/// The JSON response from calling:
/// `${smallstep_ca_url}/provisioners`
///
/// This takes a reference to a tinystep client, and provides an `Iterable`
/// over a `StepProvisioner`.
pub struct StepProvisionersPaginator<'a> {
	/// A cnt into the current fetched_last
	cnt: usize,
	/// The last fetched item.
	fetched_last: Option<StepProvisionersResponseRaw>,
	/// The underlying tinystep client to make requests with.
	tclient: &'a TinystepClient,
}

impl<'a> StepProvisionersPaginator<'a> {
	/// Construct a new paginator for `/provisioners` endpoint.
	#[must_use]
	pub fn new(client: &'a TinystepClient) -> StepProvisionersPaginator<'a> {
		Self {
			cnt: 0,
			fetched_last: None,
			tclient: client,
		}
	}
}

impl<'a> Iterator for StepProvisionersPaginator<'a> {
	type Item = Result<StepProvisioner>;

	/// Move to the next item inside of a paginator.
	fn next(&mut self) -> Option<Self::Item> {
		if self.fetched_last.is_none() {
			// This is a first fetch
			let res = crate::api::provisioners_raw(None, self.tclient);
			if let Err(res_err) = res {
				return Some(Err(res_err));
			}
			self.fetched_last = Some(res.unwrap());
		}

		let mut page = self.fetched_last.as_ref().unwrap();
		if page.provisioners.is_empty() {
			return None;
		}
		// We need to go fetch the next page.
		if self.cnt >= page.provisioners.len() {
			if page.next_cursor.is_empty() {
				return None;
			} else {
				let res =
					crate::api::provisioners_raw(Some(page.next_cursor.clone()), self.tclient);
				if let Err(res_err) = res {
					return Some(Err(res_err));
				}
				self.fetched_last = Some(res.unwrap());
				self.cnt = 0;
				page = self.fetched_last.as_ref().unwrap();
			}
		}
		// We may have fetched another page, check for emptyness again.
		if page.provisioners.is_empty() {
			return None;
		}

		// At this point we're guaranteed to be safe for indexing.
		Some(Ok(page.provisioners.get(self.cnt).unwrap().clone()))
	}
}

/// The JSON response from calling:
/// `${smallstep_ca_url}/provisioners`
///
/// This takes a reference to a tinystep client, and provides an `futures::Stream`
/// over a `StepProvisioner`.
pub struct StepProvisionersAsyncPaginator<'fetch, 'client: 'fetch> {
	/// A cnt into the current fetched_last
	cnt: usize,
	/// The last fetched item.
	fetched_last: Option<StepProvisionersResponseRaw>,
	/// An optional currently pending fetch.
	current_pending_fetch:
		Option<Pin<Box<dyn Future<Output = Result<StepProvisionersResponseRaw>> + 'fetch>>>,
	/// The underlying tinystep client to make requests with.
	tclient: &'client TinystepClient,
}

impl<'fetch, 'client: 'fetch> StepProvisionersAsyncPaginator<'fetch, 'client> {
	/// Construct a new async paginator for `/provisioners` endpoint.
	#[must_use]
	pub fn new(client: &'client TinystepClient) -> StepProvisionersAsyncPaginator<'fetch, 'client> {
		Self {
			cnt: 0,
			fetched_last: None,
			current_pending_fetch: None,
			tclient: client,
		}
	}
}

impl<'fetch, 'client: 'fetch> Stream for StepProvisionersAsyncPaginator<'fetch, 'client> {
	type Item = Result<StepProvisioner>;

	fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
		let this = unsafe { self.get_unchecked_mut() };
		if this.current_pending_fetch.is_none() && this.fetched_last.is_none() {
			this.current_pending_fetch =
				Some(crate::api::provisioners_raw_async(None, this.tclient).boxed_local());
		}

		if this.current_pending_fetch.is_some() {
			// Check if current_pending_fetch is done.
			match this.current_pending_fetch.as_mut().unwrap().poll_unpin(cx) {
				Poll::Pending => {
					return Poll::Pending;
				}
				Poll::Ready(val) => {
					if let Err(err_value) = val {
						return Poll::Ready(Some(Err(err_value)));
					} else {
						this.fetched_last = Some(val.unwrap());
					}
				}
			}
			this.current_pending_fetch = None;
		}

		let page = this.fetched_last.as_ref().unwrap();
		if page.provisioners.is_empty() {
			return Poll::Ready(None);
		}
		if this.cnt >= page.provisioners.len() {
			if page.next_cursor.is_empty() {
				return Poll::Ready(None);
			} else {
				this.current_pending_fetch = Some(
					crate::api::provisioners_raw_async(
						Some(page.next_cursor.clone()),
						this.tclient,
					)
					.boxed_local(),
				);
				this.cnt = 0;
				return Poll::Pending;
			}
		}

		Poll::Ready(Some(Ok(page.provisioners.get(this.cnt).unwrap().clone())))
	}
}
