use rosc::{OscBundle, OscMessage, OscPacket, OscType};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct TongueDir {
	pub x: f32,
	pub y: f32,
}

/// Helper struct to avoid making a new packet for every response.
// The inner value is always a bundle containing only messages.
#[derive(Debug)]
pub struct TongueOscPacket(OscPacket);

const PARAM_NAMES: &[&str] = &[
	"X",
	"X1",
	"X2",
	"X4",
	"XNegative",
	"Y",
	"Y1",
	"Y2",
	"Y4",
	"YNegative",
];

impl TongueOscPacket {
	pub fn new() -> Self {
		let content = PARAM_NAMES
			.iter()
			.map(|name| {
				OscPacket::Message(OscMessage {
					addr: format!("/avatar/parameters/FT/v2/Tongue{name}"),
					args: vec![OscType::Nil],
				})
			})
			.collect();

		Self(OscPacket::Bundle(OscBundle {
			content,
			timetag: (0, 0).into(), // vrc doesn't seem to care about this
		}))
	}

	/// Update the packet to represent this tongue direction.
	pub fn set_dir(&mut self, dir: TongueDir) {
		let OscPacket::Bundle(ref mut bundle) = self.0 else {
			unreachable!();
		};

		fn update_axis(packets: &mut [OscPacket; PARAM_NAMES.len() / 2], value: f32) {
			let binary = ((value.abs() * 8.0) as u8).min(7);
			for (i, osc_type) in packets
				.iter_mut()
				.filter_map(|p| match p {
					OscPacket::Message(msg) => Some(&mut msg.args[0]),
					_ => unreachable!(),
				})
				.enumerate()
			{
				*osc_type = match i {
					0 => OscType::Float(value),
					1 => OscType::Bool(binary & 1 != 0),
					2 => OscType::Bool(binary & 2 != 0),
					3 => OscType::Bool(binary & 4 != 0),
					4 => OscType::Bool(value.is_sign_negative()),
					_ => unreachable!(),
				};
			}
		}

		let packets = bundle.content.as_chunks_mut().0;
		update_axis(&mut packets[0], dir.x);
		update_axis(&mut packets[1], dir.y);
	}
}

impl AsRef<OscPacket> for TongueOscPacket {
	fn as_ref(&self) -> &OscPacket {
		&self.0
	}
}

impl std::borrow::Borrow<OscPacket> for TongueOscPacket {
	fn borrow(&self) -> &OscPacket {
		&self.0
	}
}
