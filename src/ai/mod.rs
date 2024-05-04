use bevy :: prelude :: *;

use polyanya :: Mesh as PolyanyaMesh;

use super :: game :: GameState;

use super :: resource_collection :: ResourceCollectionStage;

pub mod systems;

mod utils;
use utils :: *;

pub struct AiPlugin;

impl Plugin for AiPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems(Update, (
				systems::update_navmesh_obstacles,
				systems::movable_update,
				systems::movable_collision_avoidance,
				systems::give_path_on_click,
				systems::click_point_draw,
				systems::selected_path_draw,
				systems::spawn_svin_at_raypicked_base,
				systems::spawn_stresstest,
				systems::spawn_task_resource_collection,
				systems::update_task_resource_collection,
				// display_path,
			).run_if(in_state(GameState::Main)))

			.add_systems(Update, (
				systems::collect_navmesh_obstacles,
				systems::collect_spawn_locators,
				systems::collect_interact_locators,
			).run_if(in_state(GameState::Main)))
		;
	}
}

#[derive(Resource)]
pub struct PolyanyaResource {
	pub mesh : PolyanyaMesh,
}

#[derive(Component)]
pub struct NavmeshWireframe;

#[derive(Component)]
pub struct NavmeshObstacleContainer;

#[derive(Component)]
pub struct NavmeshObstacleAabb;

#[derive(Component)]
pub struct Occupied;

#[derive(Component, Deref)]
pub struct Occupies(pub Entity);

impl Occupies {
	pub fn whom(&self) -> Entity { self.0 }
}

#[derive(Component, PartialEq, Eq, Clone, Copy, Default)]
pub enum Locator {
	Spawn,
	#[default]
	Interact,
}

#[derive(Component)]
pub struct LocatorsContainer;

#[derive(Component)]
pub struct NpcSpawner;

#[derive(Component)]
pub struct NpcInteractable;

#[derive(Component)]
pub struct NpcMovable;

#[derive(Component, Default)]
pub struct NpcTaskMove {
	speed : f32,
}

#[derive(Component)]
pub struct NpcTaskMoveFinished;

#[derive(Component, Default)]
pub struct MovePath {
	current			: Vec3,
	next			: Vec<Vec3>,
	target_rotation	: Option<Quat>,
	target_entity	: Option<Entity>,
	altered			: bool,
	obstructed		: bool,
}

impl MovePath {
	pub fn new(current: Vec3, next: Vec<Vec3>, target_rotation: Option<Quat>, target_entity: Option<Entity>) -> Self {
		Self {
			current,
			next,
			target_rotation,
			target_entity,
			..default()
		}
	}
}

#[derive(Component, Default)]
pub struct NpcTaskResourceCollection {
	pub stage					: ResourceCollectionStage,
	pub resource_entity			: Option<Entity>,
}

#[derive(Component)]
pub struct NpcSpawnTaskResourceCollection {
	pub queue		: Vec::<(usize, Entity)>,
}

#[derive(Component)]
pub struct ClickPoint {
	pub init_time	: f32,
	pub duration	: f32,
}

impl ClickPoint {
	pub fn new(init_time: f32, duration: f32) -> Self {
		Self {
			init_time,
			duration
		}
	}
}
