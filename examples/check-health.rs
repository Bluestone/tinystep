use color_eyre::Result;
use tinystep::TinystepClient;

pub fn main() -> Result<()> {
	init_tracing();
	let client = TinystepClient::new_hosted("bluestone", None)?;
	println!(
		"Health is: {:?}",
		client.get::<tinystep::types::StepHealthResponse>("/health")?
	);
	Ok(())
}

fn init_tracing() {
	use tracing_subscriber::prelude::*;
	use tracing_subscriber::{fmt, EnvFilter};

	let fmt_layer = fmt::layer().with_target(false);
	let filter_layer = EnvFilter::try_from_default_env()
		.or_else(|_| EnvFilter::try_new("info"))
		.unwrap();

	tracing_subscriber::registry()
		.with(filter_layer)
		.with(fmt_layer)
		.init();
}
