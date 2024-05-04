use bevy :: {
	prelude :: *,
	pbr :: NotShadowCaster,
	render :: primitives :: Aabb,
};

use bevy_rapier3d :: prelude :: *;
use polyanya :: Triangulation;
use bevy_vector_shapes :: prelude :: *;

use super :: *;

use crate :: game :: {
	BaseBuilding,
	MainEntities,
	Raypick,
	Selected,
	spawn,
	SVIN_CARRYING_CAPACITY
};

use crate :: setup :: GROUND_HSIZE;

use crate :: assets :: GameAssets;

use crate :: handheld :: { Handheld, HandheldOwner };

use crate :: resource_collection :: {
	ResourceCollector,
	CollectableResource,
	ActiveCollecting,
};

pub fn collect_navmesh_obstacles(
		q_obstacle	: Query<Entity, Added<NavmeshObstacleContainer>>,
		q_aabb		: Query<&Aabb>,
		q_children	: Query<&Children>,
	mut commands	: Commands,
) {
	for obstacle_entity in q_obstacle.iter() {
		for descendant in q_children.iter_descendants(obstacle_entity) {
			if let Ok(_) = q_aabb.get(descendant) {
				commands.entity(descendant).insert(NavmeshObstacleAabb);
			}
		}
	}
}

pub fn collect_spawn_locators(
	mut q_container	: Query<Entity, (With<LocatorsContainer>, Added<NpcSpawner>)>,
		q_name		: Query<&Name>,
		q_children	: Query<&Children>,
	mut commands	: Commands,
) {
	for container_entity in q_container.iter_mut() {
		collect_locators_for_container(&q_children, &q_name, Locator::Spawn, container_entity, &mut commands);
	}
}

pub fn collect_interact_locators(
	mut q_container	: Query<Entity, (With<LocatorsContainer>, Added<NpcInteractable>)>,
		q_name		: Query<&Name>,
		q_children	: Query<&Children>,
	mut commands	: Commands,
) {
	for container_entity in q_container.iter_mut() {
		collect_locators_for_container(&q_children, &q_name, Locator::Interact, container_entity, &mut commands);
	}
}

pub fn update_navmesh_obstacles(
		q_navmesh_obstacle_aabb	: Query<(&GlobalTransform, &Aabb), With<NavmeshObstacleAabb>>,
		q_navmesh_obstacle_aabb_changed	: Query<Entity, Or<(Added<NavmeshObstacleAabb>, Changed<NavmeshObstacleAabb>)>>,
		q_navmesh_obstacle_aabb_removed : RemovedComponents<NavmeshObstacleAabb>,
		q_navmesh_wireframe		: Query<Entity, With<NavmeshWireframe>>,
	mut meshes					: ResMut<Assets<Mesh>>,
	mut materials				: ResMut<Assets<StandardMaterial>>,
	mut commands				: Commands
) {
	// nothing changed so nothing to do further
	if q_navmesh_obstacle_aabb_changed.is_empty() && q_navmesh_obstacle_aabb_removed.is_empty() { return }

	let mut polyanya_triangulation = Triangulation::from_outer_edges(&[
		Vec2::new(-GROUND_HSIZE, -GROUND_HSIZE),
		Vec2::new(-GROUND_HSIZE,  GROUND_HSIZE),
		Vec2::new( GROUND_HSIZE,  GROUND_HSIZE),
		Vec2::new( GROUND_HSIZE, -GROUND_HSIZE),
	]);

	let mut navmesh_obstacles_aabb = Vec::new();

	for (transform, aabb) in q_navmesh_obstacle_aabb.iter() {
		let min = transform.transform_point(aabb.min().into());
		let max = transform.transform_point(aabb.max().into());

		let v0 = Vec3::new(min.x, 0.0, min.z);
		let v1 = Vec3::new(min.x, 0.0, max.z);
		let v2 = Vec3::new(max.x, 0.0, max.z);
		let v3 = Vec3::new(max.x, 0.0, min.z);

		navmesh_obstacles_aabb.push([
			Vec2::new(v0.x, v0.z),
			Vec2::new(v1.x, v1.z),
			Vec2::new(v2.x, v2.z),
			Vec2::new(v3.x, v3.z),
		].to_vec());
	};

	polyanya_triangulation.set_unit_radius(0.4);

	polyanya_triangulation.add_obstacles(navmesh_obstacles_aabb);

	polyanya_triangulation.merge_overlapping_obstacles();

	let Some(mut navmesh) = polyanya_triangulation.as_navmesh() else { panic!("navmesh building failed!") };

	navmesh.bake();

	if let Ok(entity) = q_navmesh_wireframe.get_single() {
		commands.entity(entity).despawn_recursive();
	}

	commands.spawn((
		PbrBundle {
			mesh: meshes.add(navmesh_to_wireframe(&navmesh)),
			transform: Transform::from_translation(Vec3::new(0., 0.01, 0.)),
			material: materials.add(StandardMaterial {
				base_color: Color::GREEN,
				unlit: true,
				depth_bias: 100.0,
				..default()
			}),
			visibility: Visibility::Hidden,
			..default()
		},
		NotShadowCaster,
		NavmeshWireframe,
	));

	commands.insert_resource(PolyanyaResource { mesh: navmesh });
}

pub fn give_path_on_click(
		mouse_button	: Res<ButtonInput<MouseButton>>,
		time			: Res<Time>,
		polyanya		: Res<PolyanyaResource>,
		main_entities	: Res<MainEntities>,
		q_raypick		: Query<&Raypick>,
		q_movable		: Query<(Entity, &Transform, &HandheldOwner), (With<NpcMovable>, With<Selected>, Without<Locator>)>,
		q_locators_container : Query<Entity, (With<LocatorsContainer>, With<NpcInteractable>)>,
		q_collectable	: Query<&CollectableResource>,
		q_children		: Query<&Children>,
		q_locator		: Query<(&Locator, &GlobalTransform, Option<&Occupied>)>,
	mut q_handheld		: Query<&mut Handheld>,
	mut commands		: Commands,
) {
	let navmesh = &polyanya.mesh;

	let Ok(raypick) = q_raypick.get(main_entities.player_camera) else { panic!("player camera has no raypick!") };

	let Some(raypicked_entity) = raypick.entity else { return };

	if !mouse_button.just_pressed(MouseButton::Right) { return }

	let locators_container_result = q_locators_container.get(raypicked_entity);

	for (movable_entity, movable_transform, handheld_owner) in q_movable.iter() {
		if q_movable.get(raypicked_entity).is_ok() { continue }

		let target_position;
		let mut target_rotation = None;
		let mut target_entity = None;

		if let Ok(container_entity) = locators_container_result {
			let Some((pos, rot, entity)) = pick_nearest_locator(
				Locator::Interact,
				&container_entity,
				movable_transform,
				&q_children,
				&q_locator,
			) else { panic!("pick_nearest_locator returned None but LocatorsContainer was explicitely requested with NpcInteractable!") };

			target_position = pos;
			target_rotation = Some(rot);
			target_entity = Some(entity);

			// start resource collection task if raypicked entity is collectable
			if q_collectable.get(raypicked_entity).is_ok() {
				commands.entity(movable_entity).insert(NpcTaskResourceCollection {
					resource_entity : Some(raypicked_entity),
					..default()
				});
			}
		} else {
			target_position = raypick.pos;

			// cancel current resource collection task and deactive handheld
			commands.entity(movable_entity).remove::<NpcTaskResourceCollection>();
			if let Ok(mut handheld) = q_handheld.get_mut(handheld_owner.handheld_entity) {
				if handheld.activated() {
					handheld.deactivate();
				}
			}
		}

		let mut from3	= movable_transform.translation; from3.y = 0.0;
		let 	from2	= from3.xz();
		let 	to2		= target_position.xz();

		if !navmesh.point_in_mesh(to2) {
			println!("Error: can't create path to {:?}! clicked point is not on navmesh!", to2);
			continue;
		}

		let Some(path_wlen) = navmesh.path(from2, to2) else {
			println!("Error: can't create path from {:?} to {:?}! navmesh.path returned None!", from3, target_position);
			continue;
		};

		if let Some((first, remaining)) = path_wlen.path.split_first() {
			let mut remaining = remaining.iter().map(|v| { Vec3::new(v.x, 0.0, v.y) }).collect::<Vec<Vec3>>();
			remaining.reverse();

			let current = Vec3::new(first.x, 0.0, first.y);

			commands.entity(movable_entity).insert((
				MovePath {
					current,
					next: remaining,
					target_rotation,
					target_entity,
					..default()
				},
				NpcTaskMove {
					speed : 2.0
				}
			));

		    commands.spawn((
				SpatialBundle {
					transform : Transform::from_translation(target_position),
					..default()
				},
				ClickPoint::new(time.elapsed_seconds(), 0.7)
			));
		}
	}
}

pub fn click_point_draw(
		time			: Res<Time>,
		q_click_point	: Query<(Entity, &ClickPoint, &Transform)>,
	mut painter			: ShapePainter,
	mut commands		: Commands,
) {
	let seconds = time.elapsed_seconds();

	for (click_point_entity, click_point, transform) in q_click_point.iter() {
		let target_pos = transform.translation;
		let duration = click_point.duration;
		let elapsed_since_click = seconds - click_point.init_time;
		let progress_offset = 0.0;
		let alpha = 1.0 - (elapsed_since_click / duration);
		let color = (Color::GREEN + Color::WHITE * 0.25).with_a(alpha);
		let ease_function = interpolation::EaseFunction::CubicInOut;

		if duration > elapsed_since_click {
			draw_floor_circle(0.5, 0.15, progress_offset, &target_pos, elapsed_since_click - 0.2, duration, ease_function, color, &mut painter);
			draw_floor_circle(0.3, 0.16, progress_offset, &target_pos, elapsed_since_click - 0.1, duration, ease_function, color, &mut painter);
			draw_floor_circle(0.1, 0.17, progress_offset, &target_pos, elapsed_since_click - 0.0, duration, ease_function, color, &mut painter);
		} else {
			commands.entity(click_point_entity).despawn_recursive();
		}
	}
}

pub fn selected_path_draw(
		time			: Res<Time>,
		q_path			: Query<&MovePath, With<Selected>>,
	mut painter			: ShapePainter,
) {
	let seconds = time.elapsed_seconds();

	for path in q_path.iter() {
		let target_pos = if let Some(first) = path.next.first() { first } else { &path.current };
		let color = (Color::GREEN + Color::WHITE * 0.25).with_a(1.0);
		let ease_function = interpolation::EaseFunction::QuadraticInOut;
		let duration = 2.5;
		let progress_offset = -0.5;

		draw_floor_circle(0.7, 0.15, progress_offset, target_pos, seconds, duration, ease_function, color, &mut painter);
	}
}

pub fn movable_update(
		time			: Res<Time>,
		q_locator		: Query<&Locator>,
		q_occupies		: Query<&Occupies>,
	mut q_transform		: Query<&mut Transform>,
	mut q_movable		: Query<(Entity, &NpcTaskMove, &mut MovePath), With<NpcMovable>>,
	mut commands		: Commands,
) {
	for (movable_entity, move_task, mut path) in q_movable.iter_mut() {
		if let Ok(occupies) = q_occupies.get(movable_entity) {
			commands.entity(occupies.whom()).remove::<Occupied>();
			commands.entity(movable_entity).remove::<Occupies>();
		}

		let Ok(mut movable_transform) = q_transform.get_mut(movable_entity) else { panic!("entity with NpcTaskMove has not Transform component!") };

		let mut movable_position_navmesh = Vec3::new(movable_transform.translation.x, 0.0, movable_transform.translation.z);

		let move_direction = (path.current - movable_position_navmesh).normalize_or_zero();

		let delta = move_direction * time.delta_seconds() * move_task.speed;

		movable_transform.translation += delta;

		if !path.altered {
			movable_transform.rotation = Quat::from_rotation_arc(Vec3::Z, move_direction);
		}

		movable_position_navmesh += delta;

		let distance_to_target = movable_position_navmesh.distance(path.current);

		let distance_margin = 0.01;

		if distance_to_target < distance_margin {
			if let Some(next) = path.next.pop() {
				path.current = next;
			} else {
				commands.entity(movable_entity)
					.insert(NpcTaskMoveFinished)
					.remove::<MovePath>()
					.remove::<NpcTaskMove>()
				;

				movable_transform.translation.x = path.current.x;
				movable_transform.translation.z = path.current.z;

				if let Some(target_rotation) = path.target_rotation {
					movable_transform.rotation = target_rotation;
				}

				if let Some(target_entity) = path.target_entity {
					if let Ok(_) = q_locator.get(target_entity) {
						commands.entity(movable_entity).insert(Occupies { 0: target_entity });
						commands.entity(target_entity).insert(Occupied);
					}
				}
			}
		}
	}
}

pub fn movable_collision_avoidance(
		rapier_context	: Res<RapierContext>,
		polyanya		: Res<PolyanyaResource>,
		q_transform		: Query<&GlobalTransform>,
		q_movable		: Query<(Entity, &Collider), With<NpcTaskMove>>,
	mut q_path			: Query<&mut MovePath>,
) {
	let mut paths_to_alter = Vec::new();
	let mut paths_to_obstruct = Vec::new();

	let mut last_close_target = Vec3::ZERO;

	for (movable_entity, collider) in q_movable.iter() {
		let Ok(movable_transform) = q_transform.get(movable_entity) else { panic!("entity with NpcTaskMove has no Transform component!") };

		let Ok(path) = q_path.get(movable_entity) else { panic!("") };

		let mut movable_position_navmesh = movable_transform.translation();
		movable_position_navmesh.y = 0.0;

		let Some(cylinder) = collider.as_cylinder() else { panic!("currently npcs can only have cylinder as collider!") };
		let cylinder_diameter = cylinder.radius() * 2.0;

		let distance_to_target = movable_position_navmesh.distance(path.current);

		// check for collision only when approaching target position
		if path.next.len() != 0 || distance_to_target > cylinder_diameter {
			continue;
		}

		let fixed_target = path.target_entity.is_some();

		let position_to_test = movable_transform.translation() + Vec3::Y * 0.01;

		let mut hit_entity = None;
		let mut nearest_dist_sq = f32::MAX;

		rapier_context.intersections_with_shape(
			position_to_test,
			Quat::IDENTITY,
			collider,
			QueryFilter::new().exclude_collider(movable_entity),
			|entity: Entity| -> bool {
				let Ok(hit_entity_transform) = q_transform.get(entity) else { panic!("hit entity doesnt have transform component!") };

				let dist_sq = movable_transform.translation().distance_squared(hit_entity_transform.translation());

				if dist_sq < nearest_dist_sq {
					nearest_dist_sq = dist_sq;
					hit_entity = Some(entity);
				}

				true
			}
		);

		let Some(hit_entity) = hit_entity else {
			continue;
		};

		if fixed_target {
			if let Ok(other_path) = q_path.get(hit_entity) {
				let targets_are_close = (other_path.current - path.current).length() < cylinder_diameter;
				let last_close_target_found = (last_close_target - path.current).length() < cylinder_diameter;

				if targets_are_close && !last_close_target_found {
					last_close_target = path.current;

					continue;
				}

				last_close_target = Vec3::ZERO;
			}

			paths_to_obstruct.push(movable_entity);
		} else {
			let Ok(hit_entity_transform) = q_transform.get(hit_entity) else { panic!("hit entity doesnt have transform component!") };

			let mut hit_offset = movable_transform.translation() - hit_entity_transform.translation();
			hit_offset.y = 0.0;

			let disjoint_direction = hit_offset.normalize_or_zero();
			let disjoint_offset = (cylinder_diameter - hit_offset.length()) * disjoint_direction;

			paths_to_alter.push((movable_entity, movable_position_navmesh + disjoint_offset));
		}
	}

	for entity in paths_to_obstruct.iter() {
		if let Ok(mut path) = q_path.get_mut(*entity) {
			path.obstructed = true;
		}
	}

	for (entity, new_current) in paths_to_alter.iter() {
		if let Ok(mut path) = q_path.get_mut(*entity) {
			if polyanya.mesh.point_in_mesh(new_current.xz()) {
				path.current = *new_current;
				path.altered = true;
			}
		}
	}
}

pub fn update_task_resource_collection(
		polyanya			: Res<PolyanyaResource>,
		q_interactable_locators	: Query<(Entity, &Transform), (With<NpcInteractable>, With<LocatorsContainer>, Without<NpcMovable>, Without<Locator>)>,
		q_task_move			: Query<&MovePath, With<NpcTaskMove>>,
		q_task_move_finished: Query<&NpcTaskMoveFinished>,
		q_children			: Query<&Children>,
		q_locator			: Query<(&Locator, &GlobalTransform, Option<&Occupied>)>,
	mut q_active_collector	: Query<(&ActiveCollecting, &mut ResourceCollector)>,
	mut q_handheld			: Query<&mut Handheld>,
	mut q_task_owner		: Query<(Entity, &HandheldOwner, &Transform, &mut NpcTaskResourceCollection), Without<BaseBuilding>>,
	mut commands			: Commands,
) {
	let navmesh = &polyanya.mesh;

	for (npc_entity, handheld_owner, npc_transform, mut task) in q_task_owner.iter_mut() {
		let Ok(mut handheld) = q_handheld.get_mut(handheld_owner.handheld_entity) else { panic!("HandheldOwner entity has no Handheld component!") };

		match task.stage {
			ResourceCollectionStage::MovingToResource => {
				if q_task_move_finished.get(npc_entity).is_ok() {
					task.stage = ResourceCollectionStage::CollectingResource;

					handheld.activate();

					commands.entity(npc_entity).remove::<NpcTaskMoveFinished>();
				} else if let Ok(move_path) = q_task_move.get(npc_entity) {
					if !move_path.obstructed { continue }

					let Some(resource_entity) = task.resource_entity else { panic!("for now we always assume resource entity is there") };

					let Ok((container_entity, _)) = q_interactable_locators.get(resource_entity) else { panic!("resource_entity has either no Transform component or no LocatorsContainer component!") };

					if let Some(new_move_path) = make_path_to_nearest_locator(
						Locator::Interact,
						&container_entity,
						npc_transform,
						&q_children,
						&q_locator,
						navmesh
					) {
						commands.entity(npc_entity).insert(new_move_path);
					}
				}
			},
			ResourceCollectionStage::CollectingResource => {
				if let Ok((collecting, collector)) = q_active_collector.get(npc_entity) {
					if collecting.total_shards_num() >= collector.limit {
						task.stage = ResourceCollectionStage::MovingToBase;

						handheld.deactivate();
					}
				}
			},
			ResourceCollectionStage::MovingToBase => {
				let task_move_query_res = q_task_move.get(npc_entity);
				let task_move_finished_query_res = q_task_move_finished.get(npc_entity);

				// move task is assigned already and it hasn't finished yet so nothing to do
				if task_move_query_res.is_ok() && task_move_finished_query_res.is_err() { continue }

				// moving to base is finished, now deliver resources and start the cycle over
				if task_move_query_res.is_err() && (task_move_finished_query_res.is_ok() || q_active_collector.get(npc_entity).is_err()) {
					commands.entity(npc_entity).remove::<NpcTaskMoveFinished>();

					let Some(resource_entity) = task.resource_entity else { panic!("for now we always assume resource entity is there") };

					let Ok((container_entity, _)) = q_interactable_locators.get(resource_entity) else { panic!("resource_entity has either no Transform component or no LocatorsContainer component!") };

					if let Some(new_move_path) = make_path_to_nearest_locator(
						Locator::Interact,
						&container_entity,
						npc_transform,
						&q_children,
						&q_locator,
						navmesh
					) {
						commands.entity(npc_entity).insert((
							new_move_path,
							NpcTaskMove {
								speed : 2.0
							}
						));

						task.stage = ResourceCollectionStage::MovingToResource;
					}

					continue;
				}

				// if we're here it means npc has just finished collecting resources and needs a path to base building
				let Ok((_, mut resource_collector)) = q_active_collector.get_mut(npc_entity) else { panic!("npc has either no ResourceCollector or ActiveCollecting component but it's in ResourceCollectionStage::MovingToBase") };

				// find nearest base if no cache available
				let base_building_entity = match resource_collector.base_building_entity {
					Some(entity) => entity,
					None => {
						let (mut nearest_entity, mut nearest_distance_sq) = (None, f32::MAX);
						for (base_entity, base_transform) in q_interactable_locators.iter() {
							let distance_sq = npc_transform.translation.distance_squared(base_transform.translation);

							if distance_sq < nearest_distance_sq {
								nearest_distance_sq = distance_sq;
								nearest_entity = Some(base_entity);
							}
						}

						resource_collector.base_building_entity = nearest_entity;
						nearest_entity.unwrap()
					}
				};

				if let Some(move_path) = make_path_to_nearest_locator(
					Locator::Interact,
					&base_building_entity,
					npc_transform,
					&q_children,
					&q_locator,
					navmesh
				) {
					commands.entity(npc_entity).insert((
						move_path,
						NpcTaskMove {
							speed : 2.0
						}
					));
				}
			}
		}
	}
}

pub fn spawn_task_resource_collection(
		rapier_context	: Res<RapierContext>,
		polyanya		: Res<PolyanyaResource>,
		game_assets		: Res<GameAssets>,
		q_task			: Query<(Entity, &NpcSpawnTaskResourceCollection)>,
		q_children		: Query<&Children>,
		q_locator		: Query<(&Locator, &GlobalTransform, Option<&Occupied>)>,
	mut commands		: Commands,
) {
	for (spawner_entity, task) in q_task.iter() {
		let mut leftovers = Vec::new();

		for batch in task.queue.iter() {
			let batch_size = batch.0;
			let resource_entity = Some(batch.1);

			let mut spawned = 0 as usize;

			for locator_entity in q_children.iter_descendants(spawner_entity) {
				let Ok((locator, locator_transform_global, occupied)) = q_locator.get(locator_entity) else { continue };

				if *locator != Locator::Spawn || occupied.is_some() { continue }

				let locator_transform = locator_transform_global.compute_transform();

				let Some((svin_entity, _svin_collider)) = spawn::svin(
					&locator_transform,
					SVIN_CARRYING_CAPACITY,
					&game_assets,
					true, // with_drill
					Some(spawner_entity),
					Some(&polyanya.mesh),
					Some(&rapier_context),
					&mut commands
				) else { continue };

				let interact_container = &batch.1;

				let Some(move_path) = make_path_to_nearest_locator(
					Locator::Interact,
					interact_container,
					&locator_transform,
					&q_children,
					&q_locator,
					&polyanya.mesh
				) else { continue };

				commands.entity(svin_entity)
					.insert(NpcTaskResourceCollection {
						resource_entity,
						..default()
					})
					.insert(move_path)
					.insert(NpcTaskMove {
						speed : 2.0
					})
				;

				spawned += 1;

				if spawned >= batch_size {
					break;
				}
			}

			if spawned < batch_size {
				leftovers.push((batch_size - spawned, batch.1));
			}
		}

		if leftovers.is_empty() {
			commands.entity(spawner_entity).remove::<NpcSpawnTaskResourceCollection>();
		} else {
			commands.entity(spawner_entity).insert(NpcSpawnTaskResourceCollection { queue: leftovers });
		}
	}
}

pub fn spawn_svin_at_raypicked_base(
		key				: Res<ButtonInput<KeyCode>>,
		main_entities 	: Res<MainEntities>,
		polyanya		: Res<PolyanyaResource>,
		game_assets		: Res<GameAssets>,
		rapier_context	: Res<RapierContext>,
		q_raypick		: Query<&Raypick>,
		q_children		: Query<&Children>,
		q_locator		: Query<&Locator>,
		q_transform		: Query<&Transform>,
		q_base_building	: Query<Entity, (With<LocatorsContainer>, With<BaseBuilding>)>,
	mut commands		: Commands,
) {
	let needed_keys_pressed = key.just_pressed(KeyCode::NumpadAdd);

	if !needed_keys_pressed {
		return
	}

	let Ok(raypick) = q_raypick.get(main_entities.player_camera) else { panic!("player camera has no raypick!") };

	let Some(raypicked_entity) = raypick.entity else { return };

	let Ok(base_entity) = q_base_building.get(raypicked_entity) else { return };

	for locator_entity in q_children.iter_descendants(base_entity) {
		let Ok(locator) = q_locator.get(locator_entity) else { continue };

		if *locator != Locator::Spawn { continue }

		let Ok(locator_transform) = q_transform.get(locator_entity) else { panic!("entity with Locator::Spawn component has no Transform component!") };

		if spawn::svin(
			&locator_transform,
			SVIN_CARRYING_CAPACITY,
			&game_assets,
			true, // with_drill
			Some(base_entity),
			Some(&polyanya.mesh),
			Some(&rapier_context),
			&mut commands
		).is_none() { continue };

		break;
	}
}

pub struct StressTestCache {
	pub z_offset		: f32,
	pub z_index			: usize,
	pub flipper			: f32,
}

impl Default for StressTestCache {
	fn default() -> Self {
		Self {
			z_offset 	: 0.0,
			z_index		: 0,
			flipper 	: 1.0,
		}
	}
}

pub fn spawn_stresstest(
		key				: Res<ButtonInput<KeyCode>>,
		game_assets		: Res<GameAssets>,
	mut cache			: Local<StressTestCache>,
	mut commands		: Commands,
) {
	if !key.just_pressed(KeyCode::NumpadMultiply) {
		return
	}

	let batch_size = 20;

	let mut x_index = 1 as usize;
	let mut flipper = 1.0 as f32;

	for batch_index in 0 .. batch_size {
		let offset			= Vec3::new(20.0 * flipper * x_index as f32, 0.0, cache.z_offset);

		let tealite_pos		= Vec3::new( 3.0, 0.0, 15.0) + offset;
		let purplite_pos	= Vec3::new(-3.0, 0.0, 15.0) + offset;
		let base_pos		= Vec3::ZERO + offset;

		let tealite_entity = spawn::tealite(Transform::from_translation(tealite_pos), &game_assets, &mut commands);

		let purplite_entity = spawn::purplite(Transform::from_translation(purplite_pos), &game_assets, &mut commands);

		let base_entity = spawn::base_building(Transform::from_translation(base_pos), &game_assets, &mut commands);

		commands.entity(base_entity).insert(
			NpcSpawnTaskResourceCollection { queue: [(4, tealite_entity), (4, purplite_entity)].to_vec() }
		);

		flipper *= -1.0;

		if batch_index % 2 == 0 && batch_index != 0 {
			x_index += 1;
		}
	}

	cache.z_index		+= 1;
	cache.z_offset		+= 30.0 * cache.flipper * cache.z_index as f32;
	cache.flipper		*= -1.0;
}

pub fn display_path(
		q_path: Query<(&Transform, &MovePath)>,
	mut gizmos: Gizmos
) {
	for (transform, path) in q_path.iter() {
		let mut next = path.next.clone();
		next.reverse();

		let count = next.len() + 2;

		gizmos.linestrip_gradient(
			std::iter::once(Vec3::new(transform.translation.x, 0.1, transform.translation.z))
				.chain(std::iter::once(path.current))
				.chain(next.into_iter())
				.zip(
					(0..count).map(|i| {
						Color::hsl(120.0 - 60.0 * (i + 1) as f32 / count as f32, 1.0, 0.5)
					}),
				),
		);
	}
}