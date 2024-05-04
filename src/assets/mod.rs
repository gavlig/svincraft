use bevy :: prelude :: *;

use bevy_hanabi :: prelude :: *;

use super :: game :: GameState;

mod systems;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_systems(OnEnter(GameState::Loading), (
				systems::start_assets_loading,
			))
			.add_systems(Update, (
				systems::check_assets_loading,
				systems::check_cubemap_loading
			).run_if(in_state(GameState::Loading)))
		;
	}
}

#[derive(Resource)]
pub struct Cubemap {
	pub is_loaded		: bool,
	pub image_handle	: Handle<Image>,
	pub diffuse_handle	: Handle<Image>,
	pub specular_handle	: Handle<Image>,
}

#[derive(Resource, Default)]
pub struct GameAssets {
	pub tealite			: Handle<Scene>,
	pub tealite_shard	: Handle<Scene>,
	pub purplite		: Handle<Scene>,
	pub purplite_shard	: Handle<Scene>,
	pub base_building	: Handle<Scene>,
	pub drill_miller_falls: Handle<Scene>,
	pub svin			: Handle<Scene>,

	pub resource_drilling_effect: Handle<EffectAsset>,
	pub default_drilling_effect	: Handle<EffectAsset>,
}

impl GameAssets {
	pub fn all_scene_handhles(&self) -> Vec<Handle<Scene>> {
		let handles = [
			self.tealite.clone_weak(),
			self.tealite_shard.clone_weak(),
			self.purplite.clone_weak(),
			self.purplite_shard.clone_weak(),
			self.base_building.clone_weak(),
			self.drill_miller_falls.clone_weak(),
			self.svin.clone_weak(),
		];

		handles.to_vec()
	}
}

#[derive(Resource)]
pub struct Animations(pub Vec<Handle<AnimationClip>>);