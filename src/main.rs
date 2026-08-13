#![forbid(unsafe_code)]

mod io;
mod packet;
mod settings;

use crate::{
	io::{FaceOscMessage, FaceState, TongueState},
	packet::TongueOscPacket,
	settings::Settings,
};
use anyhow::Context;
use rosc::OscPacket;
use std::{io::Read, net::UdpSocket, process::ExitCode};

struct TongueEmulation {
	settings: Settings,
	face: FaceState,
	socket: UdpSocket,
	decode_buf: Vec<u8>,
	reusable_packet: TongueOscPacket,
	encode_buf: Vec<u8>,
	last_output: TongueState,
}

impl TongueEmulation {
	fn new() -> anyhow::Result<Self> {
		let settings = Settings::load()
			.with_context(|| format!("failed to parse arguments\n{}", Settings::USAGE))?;

		let socket = UdpSocket::bind(settings.listen_socket)
			.with_context(|| format!("failed to bind socket at {}", settings.listen_socket))?;

		Ok(TongueEmulation {
			settings,
			face: FaceState::default(),
			socket,
			decode_buf: vec![0u8; rosc::decoder::MTU],
			reusable_packet: TongueOscPacket::new(),
			encode_buf: Vec::new(),
			last_output: TongueState::default(),
		})
	}

	fn handle_osc(&mut self) -> anyhow::Result<()> {
		let (data_size, _) = self
			.socket
			.recv_from(&mut self.decode_buf)
			.with_context(|| format!("failed to read data from {}", self.settings.listen_socket))?;

		let (_, osc_packet) = rosc::decoder::decode_udp(&self.decode_buf[..data_size])
			.context("failed to decode OSC")?;

		let osc_msg = match osc_packet {
			OscPacket::Message(msg) => msg,
			OscPacket::Bundle(bundle) => {
				// vrc doesn't seem to ever send bundles
				anyhow::bail!("received bundle which is currently unimplemented:\n{bundle:?}");
			},
		};

		let face_msg = match FaceOscMessage::filter_from(&osc_msg) {
			None => return Ok(()),
			Some(msg) => {
				log::trace!("received {msg:?}");
				msg
			},
		};

		self.face.update_with(face_msg);

		match face_msg {
			FaceOscMessage::TongueOut(0.0) => {
				return self.send_tongue_update(TongueState::default());
			},
			FaceOscMessage::AvatarChange => {
				self.last_output = TongueState::default();
				return Ok(());
			},
			_ => (),
		};

		if self.face.tongue_out != 0.0 {
			self.send_tongue_update(self.face.emulate_tongue_state())?;
		}

		Ok(())
	}

	fn send_tongue_update(&mut self, state: TongueState) -> anyhow::Result<()> {
		if state == self.last_output {
			return Ok(());
		}

		log::trace!(x = state.x, y = state.y; "updating tongue");
		self.reusable_packet.set(state);
		self.encode_buf.clear();
		rosc::encoder::encode_into(self.reusable_packet.as_ref(), &mut self.encode_buf)
			.context("failed to encode packet")?;
		self.socket
			.send_to(&self.encode_buf, self.settings.send_socket)
			.with_context(|| format!("failed to send packet to {}", self.settings.send_socket))?;

		self.last_output = state;

		Ok(())
	}
}

fn main() -> ExitCode {
	log::set_max_level(log::LevelFilter::Trace); // control this from cargo features
	tinylog::setup_all().unwrap();
	if let Err(err) = try_main() {
		log::error!("{err:?}\n\nPress enter to exit...");
		let _ = std::io::stdin().read(&mut [0u8]);
		return ExitCode::FAILURE;
	}
	ExitCode::SUCCESS
}

fn try_main() -> anyhow::Result<()> {
	let mut tongue_emulation = TongueEmulation::new()?;
	log::info!("Started with {:#?}", tongue_emulation.settings);
	loop {
		if let Err(err) = tongue_emulation.handle_osc() {
			log::error!("{err:?}");
		}
	}
}
