use crate::io::TongueState;
use rosc::{OscBundle, OscMessage, OscPacket, OscType};

/// Helper struct to avoid making a new packet for every response.
// (The inner value is always a bundle containing only messages.)
#[derive(Debug)]
pub struct TongueOscPacket(OscPacket);

#[rustfmt::skip]
const PARAM_NAMES: &[&str] = &[
	"X", "X1", "X2", "X4", "XNegative",
	"Y", "Y1", "Y2", "Y4", "YNegative",
];

impl TongueOscPacket {
	pub fn new() -> Self {
		let content = PARAM_NAMES
			.iter()
			.map(|name| {
				OscPacket::Message(OscMessage {
					addr: String::from_iter(["/avatar/parameters/FT/v2/Tongue", name]),
					args: vec![OscType::Nil],
				})
			})
			.collect();

		Self(OscPacket::Bundle(OscBundle {
			content,
			timetag: (0, 0).into(), // vrc doesn't seem to care about this
		}))
	}

	/// Update the packet to represent this tongue state.
	pub fn set(&mut self, state: TongueState) {
		fn update_axis(packets: &mut [OscPacket; PARAM_NAMES.len() / 2], value: f32) {
			for (packet, new_value) in packets.iter_mut().zip({
				let binary = ((value.abs() * 8.0) as u8).min(7);
				[
					OscType::Float(value),
					OscType::Bool(binary & 1 != 0),
					OscType::Bool(binary & 2 != 0),
					OscType::Bool(binary & 4 != 0),
					OscType::Bool(value.is_sign_negative()),
				]
			}) {
				match packet {
					OscPacket::Message(msg) => msg.args[0] = new_value,
					_ => unreachable!(),
				};
			}
		}

		let OscPacket::Bundle(ref mut bundle) = self.0 else {
			unreachable!();
		};
		let packets = bundle.content.as_chunks_mut().0;
		update_axis(&mut packets[0], state.x);
		update_axis(&mut packets[1], state.y);
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
