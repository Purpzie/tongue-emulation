use rosc::{OscMessage, OscType};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FaceState {
	pub jaw_x: f32,
	pub jaw_open: f32,
	pub tongue_out: f32,
	pub lip_pucker: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FaceOscMessage {
	AvatarChange,
	TongueOut(f32),
	JawX(f32),
	JawOpen(f32),
	LipPucker(f32),
}

impl FaceState {
	pub fn update_with(&mut self, msg: FaceOscMessage) {
		match msg {
			FaceOscMessage::JawOpen(f) => self.jaw_open = f,
			FaceOscMessage::JawX(f) => self.jaw_x = f,
			FaceOscMessage::LipPucker(f) => self.lip_pucker = f,
			FaceOscMessage::TongueOut(f) => self.tongue_out = f,
			FaceOscMessage::AvatarChange => *self = Self::default(),
		}
	}
}

impl FaceOscMessage {
	pub fn filter_from(msg: OscMessage) -> Option<Self> {
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
