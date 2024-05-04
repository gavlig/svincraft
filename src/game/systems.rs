use bevy :: {
	prelude :: *,
	pbr :: NotShadowCaster,
	window :: CursorGrabMode,
};

use bevy_rapier3d :: prelude :: *;
use bevy_fps_controller :: controller :: *;
use bevy_vector_shapes :: prelude :: *;

use super :: *;

use crate :: assets :: GameAssets;

use crate :: ai :: {
	PolyanyaResource,
	Locator,
	Occupied,
};

use crate :: resource_collection :: CollectedResources;

use crate :: utils :: *;

use std :: f32 :: consts :: PI;

pub fn player_state_control(
		main_entities		: Res<MainEntities>,
		q_camera			: Query<&Camera>,
		q_window			: Query<&Window>,
		q_raypick			: Query<&Raypick>,
		q_selectable		: Query<&Selectable>,
	mut q_player_state		: Query<&mut PlayerState>,
) {
	let Ok(player_camera)	= q_camera.get(main_entities.player_camera) else { panic!("player camera is not present in game world!") };
	let window				= q_window.single();

	let Ok(raypick)			= q_raypick.get(main_entities.player_camera) else { panic!("player camera doesnt have a raypick!") };

	let Ok(mut player_state) = q_player_state.get_mut(main_entities.player) else { panic!("player has no player state!") };

	let cursor_and_camera_condition	= window.cursor.visible == false && player_camera.is_active;
	let mut selectable_picked = false;
	if let Some(entity) = raypick.entity {
		if let Ok(selectable) = q_selectable.get(entity) {
			selectable_picked = !selectable.hover_only;
		}
	}

	player_state.drilling_animation_allowed = cursor_and_camera_condition && !selectable_picked;
}

pub fn player_input_control(
		build_menu_state: ResMut<BuildMenuState>,
		q_window		: Query<&Window, Changed<Window>>,
	mut q_fps_controller: Query<&mut FpsController>,
) {
	let mut enable_input = None;
	let mut enable_movement = true;

	let window_res = q_window.get_single();
	if let Ok(window) = window_res {
		if window.cursor.visible == false {
			enable_input = Some(true);
		} else {
			enable_input = Some(false);
		}
	}

	if build_menu_state.active {
		enable_movement = false;
	}

	let Ok(mut controller) = q_fps_controller.get_single_mut() else { panic!("There is no FpsController component in world meaning player cant move!") };

	if !enable_movement {
		controller.acceleration = 0.0;
	} else {
		controller.acceleration = FpsController::default().acceleration;
	}

	if let Some(enable_input) = enable_input {
		controller.enable_input = enable_input;
	}
}

pub fn cursor_control(
		key				: Res<ButtonInput<KeyCode>>,
	mut q_window		: Query<&mut Window>,
) {
	let mut window = q_window.single_mut();
	if key.just_pressed(KeyCode::Escape) {
		if window.cursor.visible == false {
			window.cursor.grab_mode = CursorGrabMode::None;
			window.cursor.visible = true;
		} else {
			window.cursor.grab_mode = CursorGrabMode::Locked;
			window.cursor.visible = false;
		}
	}
}

pub fn toggle_orbit_camera(
		key				: Res<ButtonInput<KeyCode>>,
		main_entities	: Res<MainEntities>,
	mut q_camera		: Query<&mut Camera>,
	mut commands		: Commands,
) {
	if key.pressed(KeyCode::ShiftLeft) && key.just_pressed(KeyCode::KeyO) {
		{
			let Ok(mut player_camera) = q_camera.get_mut(main_entities.player_camera) else { panic!("player camera is not present in game world!") };

			if player_camera.is_active {
				player_camera.is_active = false;
				commands.entity(main_entities.player).remove::<LogicalPlayer>();
			} else {
				player_camera.is_active = true;
				commands.entity(main_entities.player).insert(LogicalPlayer);
			}
		}

		{
			let Ok(mut orbit_camera) = q_camera.get_mut(main_entities.orbit_camera) else { panic!("orbit camera is not present in game world!") };
			if orbit_camera.is_active {
				orbit_camera.is_active = false;
			} else {
				orbit_camera.is_active = true;
			}
		}
	}
}

pub fn camera_raypick(
		rapier_context	: Res<RapierContext>,
		q_parent		: Query<&Parent>,
	mut q_camera_raypick: Query<(&mut Raypick, &GlobalTransform), With<Camera>>,
) {
	for (mut raypick, camera_transform) in q_camera_raypick.iter_mut() {
		raypick.entity = None;

		let raycast_callback = |hit_entity: Entity, intersection: RayIntersection| -> bool {
			if raypick.to_ignore.contains(&hit_entity) { return true }

			if raypick.entity.is_none() || raypick.dist > intersection.toi {
				raypick.dist	= intersection.toi;
				raypick.pos		= intersection.point;
				raypick.nrm		= intersection.normal;

				// entity with collision is very likely to be somewhere lower in the hierarchy and for most cases we need
				// the top entity (or root ancestor) when working with raypicked_entity, so we get it by going up the chain using q_parent
				raypick.entity = Some(get_top_ancestor(hit_entity, &q_parent));
			}

			true
		};

		let cast_pos = camera_transform.translation();
		let cast_dir = camera_transform.forward();
		let cast_len = 100.0; // meters

		rapier_context.intersections_with_ray(
			cast_pos,
			cast_dir,
			cast_len,		// max_toi == ray length
			true,			// solid
			QueryFilter::new(),
			raycast_callback
		);
	}
}

pub fn selectable_control(
		mouse_button	: Res<ButtonInput<MouseButton>>,
		key				: Res<ButtonInput<KeyCode>>,
		main_entities	: Res<MainEntities>,
		q_raypick		: Query<&Raypick, Without<Selectable>>,
		q_selectable	: Query<(Entity, &Selectable, Option<&Selected>)>,
	mut commands		: Commands,
) {
	let Ok(camera_raypick) = q_raypick.get(main_entities.player_camera) else { panic!("player camera has no raypick!") };

	let raypicked_entity_opt = camera_raypick.entity;

	let just_pressed = mouse_button.just_pressed(MouseButton::Left);
	let pressed = mouse_button.pressed(MouseButton::Left) && !just_pressed;

	let remove = key.pressed(KeyCode::AltLeft);
	let add = key.pressed(KeyCode::ShiftLeft);

	for (selectable_entity, selectable, selected) in q_selectable.iter() {
		if selectable.hover_only { continue }

		let selected = selected.is_some();

		if let Some(raypicked_entity) = raypicked_entity_opt {
			if selectable_entity == raypicked_entity && (just_pressed || pressed) {
				if (add || pressed) && !selectable.hover_only && !remove && !selected {
					commands.entity(selectable_entity).insert(Selected);
				} else if remove && selected {
					commands.entity(selectable_entity).remove::<Selected>();
				}
			}
		}

		if just_pressed && !add && !remove && selected {
			commands.entity(selectable_entity).remove::<Selected>();
		}
	}
}

pub fn selectable_draw(
		mouse_button	: Res<ButtonInput<MouseButton>>,
		time			: Res<Time>,
		main_entities	: Res<MainEntities>,
		q_raypick		: Query<(&Raypick, &GlobalTransform), Without<Selectable>>,
		q_selectable	: Query<(Entity, &GlobalTransform, &Selectable, Option<&Selected>)>,
	mut painter			: ShapePainter,
) {
	fn draw_bubble(
		painter	: &mut ShapePainter,
		seconds	: f32,
		position: Vec3,
		scale	: f32,
		color	: Color,
	) {
		let seconds = seconds % PI;
		let circle_size = (seconds).powf(2.8) / 20. * scale;

		painter.thickness = f32::powf(2.5, 2.8) / 20.0 * scale - circle_size;
		painter.hollow = true;
		painter.color = (color + Color::WHITE * 0.25 + Color::WHITE * circle_size).with_a(1.0);
		painter.translate(position + Vec3::Y * circle_size * 2.0 * scale);
		painter.circle(circle_size);
	}

	fn draw_selection(
		painter				: &mut ShapePainter,
		seconds				: f32,
		selectable_transform: &GlobalTransform,
		indicator_offset	: Vec3,
		camera_transform	: &GlobalTransform,
		color				: Color,
	) {
		let rotation = calc_rotation_facing_camera(selectable_transform, camera_transform);

		painter.set_translation(selectable_transform.translation() + indicator_offset);
		painter.set_rotation(rotation);
		painter.set_scale(Vec3::splat(0.7));

		let start_pos = painter.transform;
		draw_bubble(painter, seconds + 0.5, Vec3::Y * 0.3, 0.3, color);

		painter.transform = start_pos;
		draw_bubble(painter, seconds, Vec3::X * 0.4 + Vec3::NEG_Y * 0.2, 0.2, color);

		painter.transform = start_pos;
		draw_bubble(
			painter,
			seconds + PI / 3.0,
			Vec3::NEG_X * 0.45 + Vec3::NEG_Y * 0.12,
			0.25,
			color
		);

		painter.transform = start_pos;
		draw_bubble(
			painter,
			seconds + PI / 2.0,
			Vec3::NEG_X * 0.15 + Vec3::Y * 0.2,
			0.15,
			color
		);

		painter.transform = start_pos;
		draw_bubble(
			painter,
			seconds + PI / 1.2,
			Vec3::X * 0.2 + Vec3::Y * 0.45,
			0.35,
			color
		);

		painter.transform = start_pos;

		painter.translate(Vec3::NEG_Y * 0.85);

		painter.thickness = f32::powf(2.5, 2.8) / 85.0 * 0.2;
		painter.color = (color + Color::WHITE * 0.25).with_a(1.0);

		painter.arc(0.6, -PI / 2.0, PI / 2.0);
	}

	let Ok((camera_raypick, camera_transform)) = q_raypick.get(main_entities.player_camera) else { panic!("player camera has no raypick!") };

	let seconds = time.elapsed_seconds();

	let just_pressed = mouse_button.just_pressed(MouseButton::Left);
	let pressed = mouse_button.pressed(MouseButton::Left) && !just_pressed;

	// hover
	if let Some(raypicked_entity) = camera_raypick.entity {
		if let Ok((_, selectable_transform, selectable, selected)) = q_selectable.get(raypicked_entity) {
			let color = if selectable.hover_only {
				if pressed {
					Color::hex("4c79ff").unwrap()
				} else {
					Color::hex("7df0f8").unwrap()
				}
			} else {
				if selected.is_some() {
					Color::hex("26a8ff").unwrap()
				} else {
					Color::YELLOW
				}
			};
			draw_selection(&mut painter, seconds, selectable_transform, selectable.indicator_offset, camera_transform, color);
		}
	}

	// selectable
	for (selectable_entity, selectable_transform, selectable, selected) in q_selectable.iter() {
		if selectable.hover_only { continue }

		let selected = selected.is_some();

		let hovered_entity = if let Some(raypicked_entity) = camera_raypick.entity {
			selectable_entity == raypicked_entity
		} else {
			false
		};

		if selected && !hovered_entity {
			draw_selection(&mut painter, seconds, selectable_transform, selectable.indicator_offset, camera_transform, Color::GREEN);
		}
	}
}

pub fn build_menu_control(
		key_input		: Res<ButtonInput<KeyCode>>,
		rapier_context	: Res<RapierContext>,
		polyanya		: Res<PolyanyaResource>,
		game_assets		: Res<GameAssets>,
	mut	collected_resources	: ResMut<CollectedResources>,
	mut build_menu_state: ResMut<BuildMenuState>,
		q_selected_base	: Query<Entity, (With<Selected>, With<BaseBuilding>)>,
		q_selected_other: Query<Entity, (With<Selected>, Without<BaseBuilding>)>,
		q_children		: Query<&Children>,
		q_locator		: Query<(&Locator, &GlobalTransform, Option<&Occupied>)>,
	mut commands		: Commands
) {
	let menu_allowed = !q_selected_base.is_empty() && q_selected_other.is_empty();

	// toggle text ui visibilitiy
	if key_input.just_pressed(KeyCode::KeyB) {
		build_menu_state.active ^= menu_allowed;
	}

	if !menu_allowed && build_menu_state.active {
		build_menu_state.active = false;
	}

	if !menu_allowed { return }

	if key_input.just_pressed(KeyCode::Digit1) {
		if !collected_resources.is_enough(&SVIN_PRICE) { println!("NOT ENOUGH RESOURCES COLLECTED TO BUILD SVIN!"); }

		for selected_base_entity in q_selected_base.iter() {
			if !collected_resources.is_enough(&SVIN_PRICE) { break }

			collected_resources.deduct(&SVIN_PRICE);

			for locator_entity in q_children.iter_descendants(selected_base_entity) {
				let Ok((locator, locator_transform_global, occupied)) = q_locator.get(locator_entity) else { continue };

				if *locator != Locator::Spawn || occupied.is_some() { continue }

				let locator_transform = locator_transform_global.compute_transform();

				if spawn::svin(
					&locator_transform,
					SVIN_CARRYING_CAPACITY,
					&game_assets,
					true, // with_drill
					Some(selected_base_entity),
					Some(&polyanya.mesh),
					Some(&rapier_context),
					&mut commands
				).is_some()
				{
					break;
				}
			}
		}
	}
}

pub fn build_menu_draw(
		main_entities	: Res<MainEntities>,
		build_menu_state: ResMut<BuildMenuState>,
    mut text_ui			: Query<&mut Text>,
	mut commands		: Commands
) {
	let Ok(mut text_ui) = text_ui.get_mut(main_entities.build_menu) else { panic!("MainEntities::text_ui points to non existing entity! There is no text ui!") };
	if build_menu_state.active {
		text_ui.sections[0].value = concat!(
			"[Build Menu]\n",
			"- [1] Svin: 15 purplite\n"
		).into();

		commands.entity(main_entities.build_menu).insert(Visibility::Visible);
	} else {
		commands.entity(main_entities.build_menu).insert(Visibility::Hidden);
	}
}

pub fn culling_control(
		main_entities			: Res<MainEntities>,
		q_camera				: Query<(&GlobalTransform, &Camera), Without<Culling>>,
		q_children				: Query<&Children>,
	mut q_culling				: Query<(Entity, &GlobalTransform, &mut Culling), Without<Camera>>,
	mut commands				: Commands
) {
	let camera_transform = {
		let Ok((camera_player_transform, camera_player))	= q_camera.get(main_entities.player_camera) else { panic!("player camera entity has not Camera or Transform component!") };
		let Ok((camera_orbit_transform, _camera_orbit))		= q_camera.get(main_entities.orbit_camera) else { panic!("orbit camera entity has not Camera or Transform component!") };

		if camera_player.is_active {
			camera_player_transform
		} else {
			camera_orbit_transform
		}
	};

	let camera_pos = camera_transform.translation();

	for (entity, transform, mut culling) in q_culling.iter_mut() {
		const MAX_PARTICLES_VISIBILITY_DIST_SQ : f32 = 15.0 * 15.0;
		const MAX_SHADOWS_VISIBILITY_DIST_SQ : f32 = 50.0 * 50.0;

		let dist_sq = transform.translation().distance_squared(camera_pos);

		culling.particles = dist_sq > MAX_PARTICLES_VISIBILITY_DIST_SQ;

		let perform_shadow_culling = dist_sq > MAX_SHADOWS_VISIBILITY_DIST_SQ;
		if perform_shadow_culling && !culling.shadows {
			for descendant in q_children.iter_descendants(entity) {
				commands.entity(descendant).insert(NotShadowCaster);
			}
			culling.shadows = true;
		} else if !perform_shadow_culling && culling.shadows {
			for descendant in q_children.iter_descendants(entity) {
				commands.entity(descendant).remove::<NotShadowCaster>();
			}
			culling.shadows = false;
		}

	}
}
