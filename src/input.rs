use rosc::{OscMessage, OscPacket, OscType};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct TongueInput {
	pub jaw_x: f32,
	pub jaw_open: f32,
	pub tongue_out: f32,
	pub lip_pucker: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TongueInputMessage {
	AvatarChange,
	TongueOut(f32),
	JawX(f32),
	JawOpen(f32),
	LipPucker(f32),
}

impl TongueInputMessage {
	pub fn filter_from(packet: OscPacket) -> Option<Self> {
		match packet {
			OscPacket::Message(msg) => Self::filter_from_msg(msg),
			OscPacket::Bundle(bundle) => {
				log::warn!("received bundle which is currently unimplemented: {bundle:?}");
				None
			},
		}
	}

	fn filter_from_msg(msg: OscMessage) -> Option<Self> {
		let osc_addr = msg.addr.strip_prefix("/avatar/")?;
		let osc_value = msg.args.first()?;

		if matches!(osc_value, OscType::String(_)) && osc_addr == "change" {
			return Some(Self::AvatarChange);
		}

		let &OscType::Float(float) = osc_value else {
			return None;
		};

		let vrc_msg = match osc_addr.strip_prefix("parameters/FT/v2/")? {
			"TongueOut" => Self::TongueOut(float),
			"JawX" => Self::JawX(float),
			"JawOpen" => Self::JawOpen(float),
			"LipPucker" => Self::LipPucker(float),
			_ => return None,
		};

		Some(vrc_msg)
	}
}
