use bevy :: prelude :: *;

use super :: game :: {
	GameState,
	BatchOfResources,
};

use super :: utils :: *;

pub mod systems;

pub struct ResourceCollectionPlugin;

impl Plugin for ResourceCollectionPlugin {
	fn build(&self, app: &mut App) {
		app
			.insert_resource(CollectedResources::default())
			.add_systems(Update, (
				systems::resource_collecting_control,
				systems::resource_delivery_control,
				systems::collected_resource_ui,
				systems::player_slowing_control,
			).run_if(in_state(GameState::Main)))
		;
	}
}

#[derive(Component, PartialEq, Eq, Clone, Copy, Debug)]
pub enum CollectableResource {
	Tealite,
	Purplite,
}

#[derive(Resource, Default)]
pub struct CollectedResources {
	pub purplite	: usize,
	pub tealite		: usize,
}

impl CollectedResources {
	pub fn is_enough(&self, price: &BatchOfResources) -> bool {
		price.purplite <= self.purplite &&
		price.tealite <= self.tealite
	}

	pub fn deduct(&mut self, amount: &BatchOfResources) {
		self.purplite -= amount.purplite;
		self.tealite -= amount.tealite;
	}
}

#[derive(Component, Default)]
pub struct ActiveCollecting {
	pub last_update_timestamp	: Option<f32>,
		purplite_shard_entities	: Vec<Entity>,
		tealite_shard_entities	: Vec<Entity>,
		first_shard_entity		: Option<Entity>,
}

impl ActiveCollecting {
	pub fn add_new(&mut self, resource_type: CollectableResource, entity: Entity) {
		match resource_type {
			CollectableResource::Purplite => self.purplite_shard_entities.push(entity),
			CollectableResource::Tealite => self.tealite_shard_entities.push(entity),
		}

		if self.first_shard_entity.is_none() {
			self.first_shard_entity = Some(entity);
		}
	}

	pub fn get_shard_entities(&self, resource_type: CollectableResource) -> &Vec<Entity> {
		match resource_type {
			CollectableResource::Purplite => &self.purplite_shard_entities,
			CollectableResource::Tealite => &self.tealite_shard_entities,
		}
	}

	pub fn get_first_shard_entity(&self) -> Option<Entity> {
		self.first_shard_entity
	}

	pub fn shards_num_by_type(&self, resource_type: CollectableResource) -> usize {
		match resource_type {
			CollectableResource::Purplite => self.purplite_shard_entities.len(),
			CollectableResource::Tealite => self.tealite_shard_entities.len(),
		}
	}

	pub fn total_shards_num(&self) -> usize {
		self.purplite_shard_entities.len() + self.tealite_shard_entities.len()
	}
}

#[derive(Resource)]
pub struct ResourceUiEntities {
	pub purplite		: Entity,
	pub purplite_text	: Entity,
	pub tealite			: Entity,
	pub tealite_text	: Entity,
}

#[derive(Component, Default)]
pub struct ResourceCollector {
	pub base_building_entity : Option<Entity>,
	pub limit		: usize,
}

#[derive(Default)]
pub enum ResourceCollectionStage {
	#[default]
	MovingToResource,
	CollectingResource,
	MovingToBase,
}