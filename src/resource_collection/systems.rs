use bevy :: prelude :: *;

use bevy_fps_controller :: controller :: *;

use super :: *;

use crate :: assets :: GameAssets;

use crate:: game :: {
	MainEntities,
	Raypick,
	BaseBuilding,
};

use crate :: handheld :: {
	Handheld,
	CurrentHandheld,
};

use rand :: Rng;

use std :: f32 :: consts :: { PI, TAU };

pub fn resource_collecting_control(
		main_entities		: Res<MainEntities>,
		game_assets			: Res<GameAssets>,
		time				: Res<Time>,
		q_resource_collector: Query<&ResourceCollector>,
		q_handheld_raypick	: Query<(&Raypick, &Handheld), With<CurrentHandheld>>,
		q_collectable		: Query<&CollectableResource>,
	mut	q_active_collecting	: Query<&mut ActiveCollecting>,
	mut commands			: Commands
) {
	for (raypick, handheld) in q_handheld_raypick.iter() {
		let Some(handheld_owner) = handheld.owner else { continue };

		let Ok(resource_collector) = q_resource_collector.get(handheld_owner) else { continue };

		// nothing to do without raypicked entity
		let Some(raypicked_entity) = raypick.entity else { continue };

		// nothing to do if raypicked entity is not collectable
		let Ok(collectable) = q_collectable.get(raypicked_entity) else { continue };

		if !handheld.activated() {
			if let Ok(mut active_collecting) = q_active_collecting.get_mut(handheld_owner) {
				// indicating that updates have stopped coming and next mouse click will start the timer over
				active_collecting.last_update_timestamp = None;
			}

			continue
		}

		let elapsed_seconds = time.elapsed_seconds();

		let Ok(mut active_collecting) = q_active_collecting.get_mut(handheld_owner) else {
			// initialize active collecting and return since we spawn shards only after timeout
			commands.entity(handheld_owner).insert(ActiveCollecting {
				last_update_timestamp: Some(elapsed_seconds),
				..default()
			});

			continue
		};

		let collected_shards_num = active_collecting.total_shards_num();

		// can't collect more shards than allowed by limit
		if collected_shards_num >= resource_collector.limit { continue }

		let Some(last_update_timestamp) = active_collecting.last_update_timestamp else {
			active_collecting.last_update_timestamp = Some(elapsed_seconds);

			continue
		};

		// return if update timeout hasnt finished yet
		if elapsed_seconds - last_update_timestamp < 1.0 {
			continue
		}

		let asset_to_spawn = match *collectable {
			CollectableResource::Tealite => game_assets.tealite_shard.clone_weak(),
			CollectableResource::Purplite => game_assets.purplite_shard.clone_weak(),
		};

		let mut rng = rand::thread_rng();


		let shard_rotation = Quat::from_euler(
			EulerRot::XYZ,
			rng.gen_range(0.0 .. PI / 4.0),
			rng.gen_range(0.0 .. TAU),
			rng.gen_range(0.0 .. PI / 6.0)
		);

		let shard_translation = if collected_shards_num > 0 {
			let angle_x = rng.gen_range(0.0 .. PI / 4.0);
			let angle_y = (TAU / resource_collector.limit as f32) * collected_shards_num as f32;
			let angle_z = rng.gen_range(0.0 .. PI / 6.0);
			let penta_rotation = Quat::from_euler(EulerRot::XYZ, angle_x, angle_y, angle_z);

			penta_rotation.mul_vec3(Vec3::new(0.0, 0.0, 0.05))
		} else if handheld_owner == main_entities.player {
			Vec3::new(-0.20, 0.0, -0.7)
		} else {
			Vec3::new(0.0, 0.4, -0.5)
		};

		let new_shard_entity = commands.spawn((
			Name::new("Shard"),
			SceneBundle {
				scene : asset_to_spawn,
				transform : Transform {
					translation : shard_translation,
					rotation : shard_rotation,
					..default()
				},
				..default()
			},
		)).id();

		if collected_shards_num > 0 {
			let Some(first_shard) = active_collecting.get_first_shard_entity() else { panic!("there should be at least 1 shard of type {:?}!", *collectable) };
			commands.entity(first_shard).add_child(new_shard_entity);
		} else {
			let attach_target = if handheld_owner == main_entities.player {
				main_entities.player_camera
			} else {
				handheld_owner
			};

			commands.entity(attach_target).add_child(new_shard_entity);
		}

		active_collecting.add_new(*collectable, new_shard_entity);

		active_collecting.last_update_timestamp = Some(elapsed_seconds);
	}
}

pub fn resource_delivery_control(
		q_handheld_raypick	: Query<(&Raypick, &Handheld), With<CurrentHandheld>>,
		q_active_collecting	: Query<&ActiveCollecting>,
		q_base_building		: Query<&BaseBuilding>,
	mut	collected_resources	: ResMut<CollectedResources>,
	mut commands			: Commands
) {
	for (raypick, handheld) in q_handheld_raypick.iter() {
		let Some(handheld_owner) = handheld.owner else { continue };

		let Ok(active_collecting) = q_active_collecting.get(handheld_owner) else { continue };

		// return if raypicked entity is not a base building or there is no raypicked entity
		if let Some(raypicked_entity) = raypick.entity {
			let Ok(_) = q_base_building.get(raypicked_entity) else { continue };
		} else {
			continue;
		}

		let purplite_shard_entities = active_collecting.get_shard_entities(CollectableResource::Purplite);
		let tealite_shard_entities = active_collecting.get_shard_entities(CollectableResource::Tealite);

		collected_resources.purplite += purplite_shard_entities.len();
		collected_resources.tealite += tealite_shard_entities.len();

		if let Some(first_shard_entity) = active_collecting.get_first_shard_entity() {
			commands.entity(first_shard_entity).despawn_recursive();
			commands.entity(handheld_owner).remove::<ActiveCollecting>();
		}
	}
}

pub fn player_slowing_control(
		main_entities		: Res<MainEntities>,
		time				: Res<Time>,
		q_handheld_raypick	: Query<&Raypick, With<CurrentHandheld>>,
		q_collectable		: Query<&CollectableResource>,
	mut q_fps_controller	: Query<&mut FpsController>,
) {
	let Ok(handheld_raypick) = q_handheld_raypick.get(main_entities.player_handheld) else { return };

	let Ok(mut fps_controller) = q_fps_controller.get_mut(main_entities.player) else { panic!("player entity has no FpsController component!") };

	let		cur_speed = fps_controller.walk_speed;
	let mut new_speed = FpsController::default().walk_speed;

	if let Some(raypicked_entity) = handheld_raypick.entity {
		if q_collectable.get(raypicked_entity).is_ok() {
			new_speed = 2.5;
		}
	}

	let total_time = 1.0;
	let interpolated_speed = dt_independent_lerp_f32(
		cur_speed,
		new_speed,
		1.0 / 100.0 as f32,
		time.delta_seconds(),
		total_time // in seconds
	);

	if fps_controller.walk_speed != interpolated_speed {
		fps_controller.walk_speed = interpolated_speed;
	}
}

pub fn collected_resource_ui(
		resource_icon_entities	: Res<ResourceUiEntities>,
		collected_resources		: Res<CollectedResources>,
		main_entities			: Res<MainEntities>,
		q_camera_projection		: Query<&Projection, Changed<Projection>>,
	mut q_text					: Query<&mut Text>,
	mut q_transform				: Query<&mut Transform>,
) {
	if let Ok(mut purplite_text) = q_text.get_mut(resource_icon_entities.purplite_text) {
		let text = &mut purplite_text.sections[0].value;
		let num_str = collected_resources.purplite.to_string();
		if text.as_str() != num_str.as_str() {
			text.clear();
			text.push_str(num_str.as_str());
		}
	}

	if let Ok(mut tealite_text) = q_text.get_mut(resource_icon_entities.tealite_text) {
		let text = &mut tealite_text.sections[0].value;
		let num_str = collected_resources.tealite.to_string();
		if text.as_str() != num_str.as_str() {
			text.clear();
			text.push_str(num_str.as_str());
		}
	}

	// update resource icons position only when camera projection changes (like when window gets resized for example)
	if let Ok(camera_projection) = q_camera_projection.get(main_entities.ui_camera) {
		let target_entity_z = -100.;

		// calculating frustum manually because we're going to do calculations in camera space
		let camera_frustum = match camera_projection {
			Projection::Orthographic(ortho) => calc_frustum(ortho),
			_ => panic!("this only works for orthographic projection!")
		};

		let y_top		= calc_frustum_y_border(&camera_frustum, target_entity_z, true);
		let y_bottom	= calc_frustum_y_border(&camera_frustum, target_entity_z, false);
		let x_right		= calc_frustum_x_border(&camera_frustum, target_entity_z, true);

		if let Ok(mut transform) = q_transform.get_mut(resource_icon_entities.purplite) {
			transform.translation = Vec3::new(
				x_right - 320.0,
				(y_top - y_bottom) / 2.0 - 80.0,
				target_entity_z,
			);
		}

		if let Ok(mut transform) = q_transform.get_mut(resource_icon_entities.tealite) {
			transform.translation = Vec3::new(
				x_right - 150.0,
				(y_top - y_bottom) / 2.0 - 80.0,
				target_entity_z,
			);
		}
	}
}
