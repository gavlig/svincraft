use bevy :: prelude :: *;

use bevy_rapier3d :: prelude :: *;
use bevy_hanabi :: prelude :: *;

use super :: *;

use crate :: game :: {
	MainEntities,
	PlayerState,
	Raypick,
	Culling,
};

use crate :: assets :: { GameAssets, Animations };

use crate :: resource_collection :: { CollectableResource, ActiveCollecting, ResourceCollector };

use crate :: utils :: *;

use std :: f32 :: consts :: PI;

pub fn setup_animation_player(
	mut q_handheld		: Query<&mut Handheld>,
	q_parent			: Query<&Parent>,
	q_animplayer_entity	: Query<Entity, Added<AnimationPlayer>>,
) {
	for entity in &q_animplayer_entity {
		let mut next_parent = entity;
		loop {
			if let Ok(current_parent) = q_parent.get(next_parent) {
				if let Ok(mut handheld) = q_handheld.get_mut(current_parent.get()) {
					handheld.animplayer_entity = Some(entity);
					break;
				}
				// no drill found, keep looking
				next_parent = current_parent.get();
			} else {
				break;
			}
		}
	}
}

pub fn setup_aabb(
		game_assets			: Res<GameAssets>,
	mut scenes				: ResMut<Assets<Scene>>,
	mut q_handheld			: Query<&mut Handheld, Added<Handheld>>,
) {
	let get_aabb = |world: &mut World| -> Aabb {
		let mut q_aabb = world.query::<&Aabb>();

		let Ok(aabb) = q_aabb.get_single(world) else { panic!("handheld has no aabb! Most likely an exporting issue!") };

		*aabb
	};

	let Some(gscene) = scenes.get_mut(game_assets.drill_miller_falls.clone_weak()) else { panic!("drill_miller_falls is missing!")};

	for mut handheld in q_handheld.iter_mut() {
		handheld.aabb = get_aabb(&mut gscene.world);
	}
}

pub fn setup_aim_point(
	mut q_handheld			: Query<(&mut Handheld, Entity), Added<Handheld>>,
		q_children			: Query<&Children>,
		q_name				: Query<&Name>,
		q_transform_global	: Query<&GlobalTransform>,
) {
	for (mut handheld, handheld_entity) in q_handheld.iter_mut() {
		let Ok(handheld_transform_global) = q_transform_global.get(handheld_entity) else { panic!("handheld entity must have transform!") };

		for descendant in q_children.iter_descendants(handheld_entity) {
			let Ok(name) = q_name.get(descendant) else { continue };

			if name.as_str() == "aim_point" {
				let Ok(aim_point_transform_global) = q_transform_global.get(descendant) else { panic!("handheld bone `aim_point` doesnt have GlobalTransform!") };

				let handheld_transform_global_inv = handheld_transform_global.compute_matrix().inverse();

				let object_space = handheld_transform_global_inv * aim_point_transform_global.compute_matrix();

				handheld.aim_point = Transform::from_matrix(object_space);
			}
		}
	}
}

pub fn player_animation_control(
		mouse_button	: Res<ButtonInput<MouseButton>>,
		main_entities	: Res<MainEntities>,
		q_player_state	: Query<&PlayerState>,
	mut q_handheld		: Query<&mut Handheld, With<CurrentHandheld>>,
) {
	let Ok(player_state) = q_player_state.get(main_entities.player) else { panic!("player has no player state!") };

	let animation_allowed = player_state.drilling_animation_allowed;

	// only animate handheld for player and only if it's current
	let Ok(mut handheld) = q_handheld.get_mut(main_entities.player_handheld) else { return };

	if mouse_button.just_pressed(MouseButton::Left) && animation_allowed {
		handheld.activate();
	} else if mouse_button.just_released(MouseButton::Left) {
		handheld.deactivate();
	}
}

pub fn animation_control(
		time				: Res<Time>,
		animations			: Res<Animations>,
		q_active_collector	: Query<(&ActiveCollecting, &ResourceCollector)>,
	mut q_handheld			: Query<&mut Handheld, With<CurrentHandheld>>,
	mut q_animplayer		: Query<&mut AnimationPlayer>,
) {
	for mut handheld in q_handheld.iter_mut() {
		let Some(handheld_owner) = handheld.owner else { continue };

		let Some(handheld_animplayer_entity) = handheld.animplayer_entity else { continue };

		let Ok(mut handheld_animplayer) = q_animplayer.get_mut(handheld_animplayer_entity) else { continue };

		let main_animation_handle = animations.0[0].clone_weak();
		let time_elapsed_seconds = time.elapsed_seconds();

		let mut resource_collection_allowed = true;
		if let Ok((collecting, collector)) = q_active_collector.get(handheld_owner) {
			resource_collection_allowed &= collecting.total_shards_num() < collector.limit;
		}

		let default_animspeed = if resource_collection_allowed { 1.0 } else { 0.3 };

		if handheld.just_activated() {
			if handheld_animplayer.is_paused() {
				handheld_animplayer.resume();
			} else if !handheld_animplayer.is_playing_clip(&main_animation_handle) {
				handheld_animplayer.play(main_animation_handle).repeat();
			}

			handheld_animplayer.set_speed(default_animspeed);

			handheld.anim_started_timestamp = time_elapsed_seconds;
		} else if handheld.just_deactivated() {
			handheld.anim_ended_timestamp = time_elapsed_seconds;
		} else {
			if handheld.anim_started_timestamp < handheld.anim_ended_timestamp {
				let seconds_after_deactivation = time_elapsed_seconds - handheld.anim_ended_timestamp;
				if seconds_after_deactivation < 1.0 {
					handheld_animplayer.set_speed(default_animspeed * (1.0 - seconds_after_deactivation));
				} else if !handheld_animplayer.is_paused() {
					handheld_animplayer.pause();
				}
			} else {
				handheld_animplayer.set_speed(default_animspeed);
			}
		}
	}
}

pub fn drilling_particles_control(
		game_assets				: Res<GameAssets>,
		q_active_collector		: Query<(&ActiveCollecting, &ResourceCollector)>,
		q_collectable			: Query<&CollectableResource>,
		q_culling				: Query<&Culling>,
	mut q_handheld_raypick		: Query<(Entity, &mut Handheld, &Raypick), With<CurrentHandheld>>,
	mut q_effect				: Query<(&mut Transform, &ParticleEffect, &mut EffectProperties, &mut EffectSpawner)>,
	mut commands				: Commands
) {
	for (handheld_entity, mut handheld, raypick) in q_handheld_raypick.iter_mut() {
		let Some(handheld_owner) = handheld.owner else { continue };

		let raypicked_entity = raypick.entity;

		let mut particles_allowed = raypicked_entity.is_some();

		if let Ok(culling) = q_culling.get(handheld_entity) {
			particles_allowed &= !culling.particles;
		}

		if let Ok((collecting, collector)) = q_active_collector.get(handheld_owner) {
			particles_allowed &= collecting.total_shards_num() < collector.limit;
		}

		let collectable_optional = if let Some(entity) = raypicked_entity {
			q_collectable.get(entity).ok()
		} else { None };

		if handheld.activated() && particles_allowed {
			let drilling_effect_asset = match collectable_optional {
				Some(_) => game_assets.resource_drilling_effect.clone_weak(),
				None => game_assets.default_drilling_effect.clone_weak(),
			};

			let set_effect_properties = |properties : &mut EffectProperties| {
				properties.set("normal", raypick.nrm.into());

				let tangent = Quat::from_axis_angle(raypick.nrm, PI / 10.).mul_vec3(raypick.nrm);
				properties.set("tangent", tangent.into());

				if let Some(collectable) = collectable_optional {
					let color = match collectable {
						CollectableResource::Tealite => Color::hex("a9fbff").unwrap(),
						CollectableResource::Purplite => Color::hex("f6d2fd").unwrap(),
					};
					properties.set("color", color.as_rgba_u32().into());
				}
			};

			if let Some(particles_entity) = handheld.drilling_particles_entity {
				let Ok((mut transform, effect, mut properties, mut spawner)) = q_effect.get_mut(particles_entity) else { panic!("particles entity has either no EffectProperties or Transform component!") };

				if effect.handle != drilling_effect_asset {
					commands.entity(particles_entity).despawn_recursive();
					handheld.drilling_particles_entity = None;
					continue;
				}

				set_effect_properties(&mut properties);
				transform.translation = raypick.pos;
				spawner.set_active(true);
			} else {
				let mut properties = EffectProperties::default();
				set_effect_properties(&mut properties);

				handheld.drilling_particles_entity = Some(
					commands.spawn((
						Name::new("Drilling Particles"),
						ParticleEffectBundle {
							effect		: ParticleEffect::new(drilling_effect_asset),
							transform	: Transform::from_translation(raypick.pos),
							..default()
						},
						properties,
					)).id()
				);
			}
		} else if handheld.just_deactivated() || !particles_allowed {
			if let Some(particles_entity) = handheld.drilling_particles_entity {
				let Ok((_, _, _, mut spawner)) = q_effect.get_mut(particles_entity) else { panic!("handheld.drilling_particles_entity has no EffectSpawner component!") };
				spawner.set_active(false);
			}
		}
	}
}

/// To make sure `just_activated` and `just_deactivated` stays `true` for at least a frame after it was changed
/// this system was introduced. Both fields have type u8 and get assigned value of 2 upon activation and deactivation
/// respectfully. `state_control` is scheduled in `PostUpdate` which guarantees that decrement happens only after all systems
/// in `Update` had a chance to see the `just_activated` and `just_deactivated` state of a `Handheld`
pub fn state_control(mut q_handheld : Query<&mut Handheld, Changed<Handheld>>) {
	for mut handheld in q_handheld.iter_mut() {
		handheld.just_activated_dec();
		handheld.just_deactivated_dec();
	}
}

pub fn aim_point_raypick(
		rapier_context	: Res<RapierContext>,
		q_parent		: Query<&Parent>,
	mut q_handheld_raypick : Query<(&mut Raypick, &Handheld, &GlobalTransform), With<CurrentHandheld>>,
) {
	for (mut raypick, handheld, handheld_global_transform) in q_handheld_raypick.iter_mut() {
		let Some(handheld_owner) = handheld.owner else { continue };

		let handheld_hsize = Vec3::from(handheld.aabb.half_extents);

		let aim_point_global_transform = handheld_global_transform.mul_transform(handheld.aim_point);
		let (_, aim_point_rot, aim_point_pos) = aim_point_global_transform.to_scale_rotation_translation();

		let cast_pos_offset = aim_point_rot.mul_vec3(Vec3::Y * handheld_hsize.z * 2.0);

		let cast_pos = aim_point_pos - cast_pos_offset;
		let cast_dir = aim_point_global_transform.up(); // for bones this is the axis which looks at next bone if there is one
		let cast_len = handheld_hsize.z * 3.0;

		raypick.entity = None;

		let raycast_callback = |hit_entity: Entity, intersection: RayIntersection| -> bool {
			if intersection.toi > 0.0 && (raypick.entity.is_none() || raypick.dist > intersection.toi) {
				raypick.dist	= intersection.toi;
				raypick.pos		= intersection.point;
				raypick.nrm		= intersection.normal;

				// entity with collision is very likely to be somewhere lower in the hierarchy and for most cases we need
				// the top entity (or root ancestor) when working with raypicked_entity, so we get it by going up the chain using q_parent
				raypick.entity = Some(get_top_ancestor(hit_entity, &q_parent));
			}

			true
		};

		rapier_context.intersections_with_ray(
			cast_pos,
			cast_dir,
			cast_len,		// max_toi == ray length
			true,			// solid
			QueryFilter::new().exclude_collider(handheld_owner),
			raycast_callback
		);
	}
}

pub fn collision_reaction(
		time				: Res<Time>,
		q_transform			: Query<&Transform>,
		q_active_collector	: Query<(&ActiveCollecting, &ResourceCollector)>,
		q_handheld_raypick	: Query<(&Raypick, &Handheld, &Children), With<CurrentHandheld>>,
	mut commands			: Commands,
) {
	for (raypick, handheld, handheld_children) in q_handheld_raypick.iter() {
		let Some(handheld_owner) = handheld.owner else { continue };

		let Some(handheld_gltf_root_entity) = handheld_children.first() else { panic!("handheld is expected to have at least 1 child which is gltf scene root") };

		let Ok(current_transform) = q_transform.get(*handheld_gltf_root_entity) else { panic!("handheld gltf root entity has no transform!") };

		let handheld_length = handheld.aabb.half_extents.z * 2.0;

		// don't put handheld close to picked entity if resource collection is not allowed (works only for player now)
		let mut resource_collection_allowed = true;
		if let Ok((collecting, collector)) = q_active_collector.get(handheld_owner) {
			resource_collection_allowed &= collecting.total_shards_num() < collector.limit;
		}

		let (total_time, target_offset) =
		if raypick.entity.is_some() && (raypick.dist < handheld_length || resource_collection_allowed) {
			let target_gltf_root_offset = Vec3::Z * -(raypick.dist - handheld_length);
			(0.3, target_gltf_root_offset)
		} else {
			// lerp back to identity from current offset
			(0.5, Vec3::ZERO)
		};

		let interpolated_pos = dt_independent_lerp_vec3(
			current_transform.translation,
			target_offset,
			1.0 / 100.0 as f32,
			time.delta_seconds(),
			total_time // in seconds
		);

		commands.entity(*handheld_gltf_root_entity).insert(Transform::from_translation(interpolated_pos));
	}
}
