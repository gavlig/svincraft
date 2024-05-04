use bevy :: prelude :: *;

use super :: ai;

pub mod spawn;
	mod systems;

pub struct GamePlugin;

impl Plugin for GamePlugin {
	fn build(&self, app: &mut App) {
		app
			.init_state::<GameState>()

			.insert_resource(BuildMenuState::default())

			.add_systems(PreUpdate, (
				systems::culling_control,
			).run_if(in_state(GameState::Main)))

			.add_systems(Update, (
				systems::cursor_control,
				systems::camera_raypick,
				systems::toggle_orbit_camera,
				systems::player_state_control,
				systems::player_input_control,
				systems::selectable_control.after(ai::systems::movable_update),
				systems::selectable_draw,
				systems::build_menu_control,
				systems::build_menu_draw,
			).run_if(in_state(GameState::Main)))
		;
	}
}

pub const SVIN_CARRYING_CAPACITY : usize = 3;
pub const SVIN_PRICE : BatchOfResources = BatchOfResources::new(15, 0);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, States, Default)]
pub enum GameState {
	#[default]
	Loading,
	Loaded,
	Main,
}

#[derive(Resource)]
pub struct MainEntities {
	pub player_camera	: Entity,
	pub ui_camera		: Entity,
	pub orbit_camera	: Entity,
	pub player			: Entity,
	pub player_handheld	: Entity,
	pub build_menu		: Entity,
}

#[derive(Component, Default)]
pub struct PlayerState {
	pub drilling_animation_allowed : bool,
}

#[derive(Component)]
pub struct BaseBuilding;

#[derive(Component, Default)]
pub struct Raypick {
	pub entity		: Option<Entity>,
	pub to_ignore	: Vec<Entity>,
	pub pos			: Vec3,
	pub nrm			: Vec3,
	pub dist		: f32,
}

#[derive(Component, Default)]
pub struct Selectable {
	pub hover_only	: bool,
	pub indicator_offset : Vec3,
}

#[derive(Component)]
pub struct Selected;

#[derive(Component, Default)]
pub struct Culling {
	pub particles	: bool,
	pub shadows		: bool,
}

#[derive(Resource, Default)]
pub struct BuildMenuState {
	pub active		: bool,
}

pub struct BatchOfResources {
	pub purplite	: usize,
	pub tealite		: usize,
}

impl BatchOfResources {
	pub const fn new(purplite: usize, tealite: usize) -> Self {
		Self {
			purplite,
			tealite
		}
	}
}
