#![forbid(unsafe_code)]

mod input;
mod output;
mod settings;

use crate::{
	input::{TongueInput, TongueInputMessage},
	output::{TongueDirection, TongueEmulationPacket},
	settings::Settings,
};
use anyhow::Context;
use std::{io::Read, net::UdpSocket, process::ExitCode};

fn main() -> ExitCode {
	log::set_max_level(log::LevelFilter::Trace); // control this from cargo features
	log::set_boxed_logger(Box::new(tinylog::Logger::default())).unwrap();
	if let Err(err) = try_main() {
		log::error!("{err:?}\n\nPress enter to exit...");
		let _ = std::io::stdin().read(&mut [0u8]);
		return ExitCode::FAILURE;
	}
	ExitCode::SUCCESS
}

fn try_main() -> anyhow::Result<()> {
	let settings = Settings::load()
		.context("failed to parse arguments\nusage: [listening socket] [sending socket]")?;

	log::debug!("{settings:#?}");

	let socket = UdpSocket::bind(settings.listen_socket)
		.with_context(|| format!("failed to create socket at {}", settings.listen_socket))?;

	let mut global_state = GlobalState {
		settings,
		socket,
		packet: TongueEmulationPacket::new(),
		decode_buf: vec![0u8; rosc::decoder::MTU],
		input: TongueInput::default(),
		last_output: TongueDirection::default(),
	};

	log::info!("Starting");

	loop {
		if let Err(err) = global_state.handle_osc() {
			log::error!("{err:?}");
		}
	}
}

struct GlobalState {
	settings: Settings,
	socket: UdpSocket,
	packet: TongueEmulationPacket,
	decode_buf: Vec<u8>,
	input: TongueInput,
	last_output: TongueDirection,
}

impl GlobalState {
	fn handle_osc(&mut self) -> anyhow::Result<()> {
		let (data_size, _) = self
			.socket
			.recv_from(&mut self.decode_buf)
			.context("failed to receive data from socket")?;

		let (_, incoming_packet) = rosc::decoder::decode_udp(&self.decode_buf[..data_size])
			.context("failed to decode OSC from data")?;

		let msg = match TongueInputMessage::filter_from(incoming_packet) {
			None => return Ok(()),
			Some(msg) => {
				log::trace!("received {msg:?}");
				msg
			},
		};

		match msg {
			TongueInputMessage::JawOpen(f) => self.input.jaw_open = f,
			TongueInputMessage::JawX(f) => self.input.jaw_x = f,
			TongueInputMessage::LipPucker(f) => self.input.lip_pucker = f,
			TongueInputMessage::TongueOut(f) => {
				self.input.tongue_out = f;
				if f == 0.0 {
					return self.update_tongue(TongueDirection::default());
				}
			},
			TongueInputMessage::AvatarChange => {
				self.input = TongueInput::default();
				return Ok(());
			},
		};

		if self.input.tongue_out == 0.0 {
			return Ok(());
		}

		let mut dir = TongueDirection {
			x: self.input.jaw_x * 4.0,
			y: self.input.lip_pucker * 1.333 - self.input.jaw_open * 2.0,
		};

		dir.x = dir.x.clamp(-1.0, 1.0) * self.input.tongue_out;
		dir.y = dir.y.clamp(-1.0, 1.0) * self.input.tongue_out;
		if self.last_output != dir {
			self.last_output = dir;
			self.update_tongue(dir)?;
		}

		Ok(())
	}

	fn update_tongue(&mut self, dir: TongueDirection) -> anyhow::Result<()> {
		log::trace!(x = dir.x, y = dir.y; "updating tongue");
		self.packet.set_dir(dir);
		let outgoing_msg =
			rosc::encoder::encode(self.packet.as_ref()).context("failed to encode packet")?;
		self.socket
			.send_to(&outgoing_msg, self.settings.send_socket)
			.with_context(|| format!("failed to send packet to {}", self.settings.send_socket))?;
		Ok(())
	}
}
