use rosc::{OscMessage, OscType};

/// Specific kinds of [`OscMessage`] received from VRChat.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FaceOscMessage {
	JawOpen(f32),
	JawX(f32),
	LipPucker(f32),
	TongueOut(f32),
	PuppetX(f32),
	PuppetY(f32),
	PuppetActive(bool),
	AvatarChange,
}

impl FaceOscMessage {
	pub fn filter_from(msg: &OscMessage) -> Option<Self> {
		Some(match msg.args.first()? {
			&OscType::Float(float) => match msg.addr.strip_prefix("/avatar/parameters/")? {
				"FT/v2/JawOpen" => Self::JawOpen(float),
				"FT/v2/JawX" => Self::JawX(float),
				"FT/v2/LipPucker" => Self::LipPucker(float),
				"FT/v2/TongueOut" => Self::TongueOut(float),
				"TongueEmulation/PuppetX" => Self::PuppetX(float),
				"TongueEmulation/PuppetY" => Self::PuppetY(float),
				_ => return None,
			},
			&OscType::Bool(bool)
				if &msg.addr == "/avatar/parameters/TongueEmulation/PuppetActive" =>
			{
				Self::PuppetActive(bool)
			},
			OscType::String(_) if &msg.addr == "/avatar/change" => Self::AvatarChange,
			_ => return None,
		})
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct TongueState {
	/// Left and right.
	pub x: f32,
	/// Up and down.
	pub y: f32,
	// /// How far out the tongue is.
	// pub out: f32,
}

/// The current state of the user's face.
///
/// This is a cache and must be [updated](Self::update_with) after receiving a [`FaceOscMessage`].
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FaceState {
	pub jaw_open: f32,
	pub jaw_x: f32,
	pub lip_pucker: f32,
	pub tongue_out: f32,
	pub puppet: Option<TongueState>,
}

impl FaceState {
	pub fn update_with(&mut self, msg: FaceOscMessage) {
		match msg {
			FaceOscMessage::JawOpen(float) => self.jaw_open = float,
			FaceOscMessage::JawX(float) => self.jaw_x = float,
			FaceOscMessage::LipPucker(float) => self.lip_pucker = float,
			FaceOscMessage::TongueOut(float) => self.tongue_out = float,
			FaceOscMessage::PuppetX(float) => {
				if let Some(ref mut puppet) = self.puppet {
					puppet.x = float;
				}
			},
			FaceOscMessage::PuppetY(float) => {
				if let Some(ref mut puppet) = self.puppet {
					puppet.y = float;
				}
			},
			FaceOscMessage::PuppetActive(bool) => self.puppet = bool.then(TongueState::default),
			FaceOscMessage::AvatarChange => *self = Self::default(),
		}
	}

	pub fn emulate_tongue_state(&self) -> TongueState {
		match self.puppet {
			None => {
				let mut state = TongueState {
					x: self.jaw_x * 4.0,
					y: self.lip_pucker * 1.333 - self.jaw_open * 2.0,
				};

				state.x = state.x.clamp(-1.0, 1.0) * self.tongue_out;
				state.y = state.y.clamp(-1.0, 1.0) * self.tongue_out;

				state
			},
			Some(puppet) => TongueState {
				x: puppet.x * self.tongue_out,
				y: puppet.y * self.tongue_out,
			},
		}
	}
}
