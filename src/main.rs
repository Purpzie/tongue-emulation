#![forbid(unsafe_code)]

mod input;
mod output;
mod settings;

use crate::{
	input::{FaceOscMessage, FaceState},
	output::{TongueDir, TongueOscPacket},
	settings::Settings,
};
use anyhow::Context;
use rosc::OscPacket;
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

struct TongueEmulation {
	settings: Settings,
	face: FaceState,
	socket: UdpSocket,
	reusable_packet: TongueOscPacket,
	decode_buf: Vec<u8>,
	encode_buf: Vec<u8>,
	last_output: TongueDir,
}

fn try_main() -> anyhow::Result<()> {
	let settings = Settings::load()
		.context("failed to parse arguments\nusage: [listening socket] [sending socket]")?;

	log::debug!("{settings:#?}");

	let socket = UdpSocket::bind(settings.listen_socket)
		.with_context(|| format!("failed to create socket at {}", settings.listen_socket))?;

	let mut tongue_emulation = TongueEmulation {
		settings,
		socket,
		face: FaceState::default(),
		reusable_packet: TongueOscPacket::new(),
		last_output: TongueDir::default(),
		decode_buf: vec![0u8; rosc::decoder::MTU],
		encode_buf: Vec::new(),
	};

	log::info!("Started");

	loop {
		if let Err(err) = tongue_emulation.handle_osc() {
			log::error!("{err:?}");
		}
	}
}

impl TongueEmulation {
	fn handle_osc(&mut self) -> anyhow::Result<()> {
		let (data_size, _) = self
			.socket
			.recv_from(&mut self.decode_buf)
			.context("failed to receive data from socket")?;

		let (_, incoming_packet) = rosc::decoder::decode_udp(&self.decode_buf[..data_size])
			.context("failed to decode OSC")?;

		let osc_msg = match incoming_packet {
			OscPacket::Message(msg) => msg,
			OscPacket::Bundle(bundle) => {
				// vrc doesn't seem to ever send bundles
				anyhow::bail!("received bundle which is currently unimplemented:\n{bundle:?}")
			},
		};

		let face_msg = match FaceOscMessage::filter_from(osc_msg) {
			None => return Ok(()),
			Some(msg) => {
				log::trace!("received {msg:?}");
				msg
			},
		};

		self.face.update_with(face_msg);

		match face_msg {
			FaceOscMessage::TongueOut(0.0) => {
				return self.send_tongue_update(TongueDir::default());
			},
			FaceOscMessage::AvatarChange => {
				self.last_output = TongueDir::default();
				return Ok(());
			},
			_ => (),
		};

		if self.face.tongue_out == 0.0 {
			return Ok(());
		}

		let mut dir = TongueDir {
			x: self.face.jaw_x * 4.0,
			y: self.face.lip_pucker * 1.333 - self.face.jaw_open * 2.0,
		};

		dir.x = dir.x.clamp(-1.0, 1.0) * self.face.tongue_out;
		dir.y = dir.y.clamp(-1.0, 1.0) * self.face.tongue_out;

		self.send_tongue_update(dir)?;

		Ok(())
	}

	fn send_tongue_update(&mut self, dir: TongueDir) -> anyhow::Result<()> {
		if dir == self.last_output {
			return Ok(());
		}
		log::trace!(x = dir.x, y = dir.y; "updating tongue");
		self.reusable_packet.set_dir(dir);
		self.encode_buf.clear();
		rosc::encoder::encode_into(self.reusable_packet.as_ref(), &mut self.encode_buf)
			.context("failed to encode packet")?;
		self.socket
			.send_to(&self.encode_buf, self.settings.send_socket)
			.with_context(|| format!("failed to send packet to {}", self.settings.send_socket))?;
		self.last_output = dir;
		Ok(())
	}
}
