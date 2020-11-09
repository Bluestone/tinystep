use color_eyre::Result;
use std::{env::args as arg_iter, path::PathBuf};
use tinystep::TinystepClient;

pub fn main() -> Result<()> {
	init_tracing();
	let args = arg_iter().collect::<Vec<String>>();
	let client = if args.len() == 3 || args.len() == 4 {
		println!("Using Client Identity!");
		TinystepClient::new_from_hosted_with_identity(
			"bluestone",
			Some("certs".to_owned()),
			PathBuf::from(args.get(1).unwrap()),
			PathBuf::from(args.get(2).unwrap()),
			if args.len() == 3 {
				None
			} else {
				Some(args[3].to_owned())
			},
		)?
	} else {
		TinystepClient::new_from_hosted("bluestone", Some("certs".to_owned()))?
	};
	println!("Health is: {:?}", tinystep::api::health(&client)?,);
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
