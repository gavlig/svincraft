use bevy :: prelude :: *;

use bevy_rapier3d :: prelude :: *;

use polyanya :: Mesh as PolyanyaMesh;

use super :: {
	BaseBuilding,
	Selectable,
	Raypick,
	Culling,
};

use crate :: resource_collection :: { ResourceCollector, CollectableResource };

use crate :: assets :: GameAssets;

use crate :: ai :: NpcMovable;

use crate :: handheld :: {
	Handheld,
	HandheldOwner,
	CurrentHandheld,
};

use crate :: ai :: { NavmeshObstacleContainer, LocatorsContainer, NpcInteractable, NpcSpawner };

use std :: f32 :: consts :: PI;

pub fn svin(
	spawn_locator	: &Transform,
	collection_limit: usize,
	game_assets		: &GameAssets,
	with_drill		: bool,
	base_entity		: Option<Entity>,
	navmesh			: Option<&PolyanyaMesh>,
	rapier_context	: Option<&RapierContext>,
	commands		: &mut Commands,
) -> Option<(Entity, Collider)> {
	let cylinder_half_height = 0.6;
	let cylinder_radius = 0.5;

	let spawn_position = spawn_locator.translation + Vec3::Y * (cylinder_half_height + 0.01);

	let svin_collider = Collider::cylinder(cylinder_half_height, cylinder_radius);

	if let Some(rapier) = rapier_context {
		if let Some(_) = rapier.intersection_with_shape(
			spawn_position,
			spawn_locator.rotation,
			&svin_collider,
			QueryFilter::new(),
		) {
			// println!("skipping spawning because spawn point is occupied! {:?} {:?}", spawn_position, entity);
			return None;
		}
	}

	if let Some(navmesh) = navmesh {
		if !navmesh.point_in_mesh(Vec2::new(spawn_position.x, spawn_position.z)) {
			// println!("skipping spawning because spawn point is not on navmesh! occupied! {:?}", spawn_position);
			return None;
		}
	}

	let svin_entity = commands.spawn((
		Name::new("Svin Bot"),
		SceneBundle {
			scene : game_assets.svin.clone_weak(),
			transform: Transform {
				translation : spawn_position,
				rotation : spawn_locator.rotation,
				..default()
			},
			..default()
		},
		NpcMovable,
		ResourceCollector { limit : collection_limit, base_building_entity : base_entity },
		Selectable { indicator_offset : Vec3::Y, ..default() },
		Culling::default(),
		RigidBody::Fixed,
		svin_collider.clone(),
	)).id();

	if with_drill {		
		let drill_transform = Transform {
			translation: Vec3::new(-0.25, 0.0, 0.7),
			rotation: Quat::from_euler(EulerRot::XYZ, 0.1, PI + 0.1, -1.3),
			..default()
		};

		let drill_entity = drill(
			&drill_transform,
			Some(svin_entity),
			game_assets,
			commands
		);

		commands.entity(svin_entity)
			.add_child(drill_entity)
			.insert(HandheldOwner { handheld_entity: drill_entity });
	}
	
	Some((svin_entity, svin_collider))
}

pub fn tealite(
	transform	: Transform,
	game_assets	: &GameAssets,
	commands	: &mut Commands
) -> Entity {
	commands.spawn((
		Name::new("Tealite"),
		CollectableResource::Tealite,
		SceneBundle {
			scene : game_assets.tealite.clone(),
			transform,
			..default()
		},
		RigidBody::Fixed,
		AsyncSceneCollider::default(),
		NavmeshObstacleContainer,
		Selectable { hover_only : true, indicator_offset : Vec3::Y * 1.5, ..default() },
		NpcInteractable,
		LocatorsContainer,
		Culling::default(),
	)).id()
}

pub fn purplite(
	transform	: Transform,
	game_assets	: &GameAssets,
	commands	: &mut Commands
) -> Entity {
	commands.spawn((
		Name::new("Purplite"),
		CollectableResource::Purplite,
		SceneBundle {
			scene : game_assets.purplite.clone(),
			transform,
			..default()
		},
		RigidBody::Fixed,
		AsyncSceneCollider::default(),
		NavmeshObstacleContainer,
		Selectable { hover_only : true, indicator_offset : Vec3::Y * 2.0, ..default() },
		NpcInteractable,
		LocatorsContainer,
		Culling::default(),
	)).id()
}

pub fn base_building(
	transform	: Transform,
	game_assets	: &GameAssets,
	commands	: &mut Commands,
) -> Entity {
	commands.spawn((
		Name::new("Base Building"),
		BaseBuilding,
		NpcInteractable,
		SceneBundle {
			scene : game_assets.base_building.clone(),
			transform,
			..default()
		},
		RigidBody::Fixed,
		AsyncSceneCollider::default(),
		NavmeshObstacleContainer,
		NpcSpawner,
		LocatorsContainer,
		Selectable { hover_only : false, indicator_offset : Vec3::Y * 6.5, ..default() },
		Culling::default(),
	)).id()
}

pub fn drill(
	transform		: &Transform,
	owner_entity	: Option<Entity>,
	game_assets		: &GameAssets,
	commands		: &mut Commands,
) -> Entity {
	commands.spawn((
		Name::new("Drill Miller Falls"),
		SceneBundle {
			scene : game_assets.drill_miller_falls.clone_weak(),
			transform: *transform,
			..default()
		},
		Handheld::new(owner_entity),
		CurrentHandheld,
		Raypick::default(),
		Culling::default(),
	)).id()
}