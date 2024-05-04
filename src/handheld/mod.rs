use bevy :: {
	prelude :: *,
	render :: primitives :: Aabb,
	transform :: TransformSystem,
};

use super :: game :: GameState;

pub mod systems;

pub struct HandheldPlugin;

impl Plugin for HandheldPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems(Update, (
				systems::setup_aabb,
				systems::setup_animation_player,
				systems::player_animation_control,
				systems::animation_control,
				systems::drilling_particles_control,
				systems::aim_point_raypick.after(bevy_fps_controller::controller::fps_controller_render),
				systems::collision_reaction,
			).run_if(in_state(GameState::Main)))

			.add_systems(PostUpdate, (
				systems::state_control,
				systems::setup_aim_point.after(TransformSystem::TransformPropagate),
			).run_if(in_state(GameState::Main)))
		;
	}
}

#[derive(Component, Default)]
pub struct Handheld {
	pub owner					: Option<Entity>,
	pub aabb					: Aabb,
	pub aim_point				: Transform,
	pub animplayer_entity		: Option<Entity>,
	pub anim_started_timestamp	: f32,
	pub anim_ended_timestamp	: f32,
	pub drilling_particles_entity : Option<Entity>,

	activated					: bool,
	just_activated				: u8,
	just_deactivated			: u8,
}

impl Handheld {
	pub fn new(owner: Option<Entity>) -> Self {
		Self { owner, ..default() }
	}

	pub fn activated(&self) -> bool {
		self.activated
	}

	pub fn just_activated(&self) -> bool {
		self.just_activated > 0
	}

	pub fn just_deactivated(&self) -> bool {
		self.just_deactivated > 0
	}

	pub fn just_activated_dec(&mut self) {
		if self.just_activated > 0 {
			self.just_activated -= 1;
		}
	}

	pub fn just_deactivated_dec(&mut self) {
		if self.just_deactivated > 0 {
			self.just_deactivated -= 1;
		}
	}

	pub fn activate(&mut self) {
		if !self.activated {
			self.activated = true;
			self.just_activated = 2;
		}
	}

	pub fn deactivate(&mut self) {
		if self.activated {
			self.activated = false;
			self.just_deactivated = 2;
		}
	}
}

#[derive(Component)]
pub struct CurrentHandheld;

#[derive(Component)]
pub struct HandheldOwner {
	pub handheld_entity : Entity,
}