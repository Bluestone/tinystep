//! API is a module containing the raw API calls for interacting with smallstep.
//!
//! This is meant to be as close to a one to one mapping of the API exposed by
//! a smallstep server, with no additional changes. For example if you wanted
//! to call `{smallstep_api_server}/version` you can call: `api::version(&client)`.

use crate::{
	types::{StepHealthResponse, StepVersionResponse},
	TinystepClient,
};
use color_eyre::Result;
use tracing::instrument;

pub mod root;

/// Get the health status for a particular smallstep server.
///
/// Really although there is a response type, you don't need to check it,
/// as if the server responds it will always respond healthy. You can just
/// really call: `api::health(&client)?;` and get the same behavior.
///
/// If you need an async version of this method call: `health_async`.
#[instrument]
pub fn health(client: &TinystepClient) -> Result<StepHealthResponse> {
	client.get::<StepHealthResponse>("/health")
}

/// Get the health status for a particular smallstep server.
///
/// Really although there is a response type, you don't need to check it,
/// as if the server responds it will always respond healthy. You can just
/// really call: `api::health_async(&client).await?;` and get the same behavior.
#[instrument]
pub async fn health_async(client: &TinystepClient) -> Result<StepHealthResponse> {
	client.get_async::<StepHealthResponse>("/health").await
}

/// Get the version information for the server you're talking too.
///
/// Ideally you don't need to call this manually as the version is stored as
/// part of the TinystepClient, and will appear in logs.
///
/// If you need an async version of this method call: `version_async`.
#[instrument]
pub fn version(client: &TinystepClient) -> Result<StepVersionResponse> {
	client.get::<StepVersionResponse>("/version")
}

/// Get the version information for the server you're talking too,
/// asynchronously.
///
/// Ideally you don't need to call this manually as the version is stored as
/// part of the TinystepClient, and will appear in logs.
#[instrument]
pub async fn version_async(client: &TinystepClient) -> Result<StepVersionResponse> {
	client.get_async::<StepVersionResponse>("/version").await
}
