// I'd use `clap`, but that literally doubles the size of the executable.

use anyhow::Context;
use std::net::{Ipv4Addr, SocketAddr};

#[derive(Debug)]
#[non_exhaustive]
pub struct Settings {
	pub listen_socket: SocketAddr,
	pub send_socket: SocketAddr,
}

impl Settings {
	pub const USAGE: &str = "usage: [listening socket] [sending socket]";

	/// Load settings from arguments.
	pub fn load() -> anyhow::Result<Self> {
		let mut args = std::env::args().skip(1).map(|s| s.parse::<SocketAddr>());
		let listen_socket = args
			.next()
			.transpose()
			.context("invalid listening socket")?
			.unwrap_or((Ipv4Addr::LOCALHOST, 9001).into());
		let send_socket = args
			.next()
			.transpose()
			.context("invalid sending socket")?
			.unwrap_or((Ipv4Addr::LOCALHOST, 9000).into());
		Ok(Self {
			listen_socket,
			send_socket,
		})
	}
}
