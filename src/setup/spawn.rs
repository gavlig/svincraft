use bevy :: {
	prelude :: *,
	render :: {
		camera :: Exposure,
		view :: { ColorGrading, RenderLayers },
	},
	core_pipeline :: {
		// Skybox,
		bloom :: BloomSettings,
		tonemapping :: Tonemapping,
	},
};

use bevy_rapier3d :: prelude :: *;
use bevy_fps_controller :: controller :: *;
use bevy_panorbit_camera :: PanOrbitCamera;
use bevy_scene_hook :: { SceneHook, HookedSceneBundle };

use super :: GROUND_SIZE;

use crate :: game :: {
	PlayerState,
	Raypick,
};

use crate :: game :: spawn as game_spawn;

use crate :: assets :: {
	Cubemap,
	GameAssets,
};

use crate :: resource_collection :: {
	ResourceCollector,
	ResourceUiEntities
};

use std :: f32 :: consts :: PI;

pub const PLAYER_SPAWN_POINT: Vec3 = Vec3::new(0.0, 1.0, 20.0);

pub fn light(commands: &mut Commands) {
	commands.spawn((
		DirectionalLightBundle {
			directional_light : DirectionalLight {
				illuminance : light_consts::lux::CLEAR_SUNRISE,
				shadows_enabled : true,
				..default()
			},
			transform : Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
			..default()
		},
		RenderLayers::layer(0)
	));

	commands.spawn((
		DirectionalLightBundle {
			directional_light : DirectionalLight {
				illuminance : light_consts::lux::CLEAR_SUNRISE,
				shadows_enabled : true,
				..default()
			},
			transform : Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 5.3, -0.2, 0.0)),
			..default()
		},
		RenderLayers::layer(1)
	));
}

pub fn ground_plane(
	meshes		: &mut Assets<Mesh>,
	materials	: &mut Assets<StandardMaterial>,
	commands	: &mut Commands
) {
	commands.spawn((
		Name::new("Ground Plane"),
		PbrBundle {
			mesh : meshes.add(Plane3d::default().mesh().size(GROUND_SIZE, GROUND_SIZE)),
			material : materials.add(Color::GRAY),
			..default()
		},
		RigidBody::Fixed,
		Collider::halfspace(Vec3::Y).unwrap(),
	));
}

pub fn resource_ui(
	game_assets	: &GameAssets,
	commands	: &mut Commands
) -> ResourceUiEntities {
	let purplite_icon_entity = commands.spawn((
		Name::new("Purplite Icon"),
		HookedSceneBundle {
			scene: SceneBundle {
				scene : game_assets.purplite.clone(),
				transform : Transform {
					rotation: Quat::from_rotation_y(-0.3),
					scale: Vec3::splat(10.0),
					..default()
				},
				..default()
			},
			hook: SceneHook::new(|_, cmds| {
				cmds.insert(RenderLayers::layer(1));
			})
		},
		RenderLayers::layer(1),
	)).id();

	let tealite_icon_entity = commands.spawn((
		Name::new("Tealite Icon"),
		HookedSceneBundle {
			scene: SceneBundle {
				scene : game_assets.tealite.clone(),
				transform : Transform {
					rotation: Quat::from_rotation_y(-1.7),
					scale: Vec3::splat(10.0),
					..default()
				},
				..default()
			},
			hook: SceneHook::new(|_, cmds| {
				cmds.insert(RenderLayers::layer(1));
			})
		},
		RenderLayers::layer(1),
	)).id();

	let purplite_text_entity = commands.spawn(
		TextBundle::from_section(
			"0",
			TextStyle {
				font_size: 10.0,
				color: Color::hex("bdbdbd").unwrap(),
				..default()
			},
		)
		.with_style(Style {
			position_type: PositionType::Absolute,
			top: Val::Px(20.0),
			right: Val::Px(115.0),
			min_width: Val::Px(33.0),
			..default()
		}),
	).id();

	let tealite_text_entity = commands.spawn(
		TextBundle::from_section(
			"0",
			TextStyle {
				font_size: 10.0,
				color: Color::hex("bdbdbd").unwrap(),
				..default()
			},
		)
		.with_style(Style {
			position_type: PositionType::Absolute,
			top: Val::Px(20.0),
			right: Val::Px(10.0),
			min_width: Val::Px(33.0),
			..default()
		}),
	).id();

	ResourceUiEntities {
		purplite		: purplite_icon_entity,
		purplite_text	: purplite_text_entity,
		tealite			: tealite_icon_entity,
		tealite_text	: tealite_text_entity,
	}
}

pub fn player_entity(commands	: &mut Commands) -> Entity {
	commands.spawn((
		Collider::capsule(Vec3::Y * 0.5, Vec3::Y * 1.0, 0.5),
		Friction {
			coefficient: 0.0,
			combine_rule: CoefficientCombineRule::Min,
		},
		Restitution {
			coefficient: 0.0,
			combine_rule: CoefficientCombineRule::Min,
		},
		ActiveEvents::COLLISION_EVENTS,
		Velocity::zero(),
		RigidBody::Dynamic,
		Sleeping::disabled(),
		LockedAxes::ROTATION_LOCKED,
		AdditionalMassProperties::Mass(1.0),
		GravityScale(0.0),
		Ccd { enabled: true }, // Prevent clipping when going fast
		LogicalPlayer,
		FpsControllerInput {
			pitch: 0.0,
			yaw: 0.0,
			..default()
		},
		FpsController {
			air_acceleration: 80.0,
			..default()
		},
	))
	.insert(PlayerState::default())
	.insert(ResourceCollector { limit : 5, ..default() } )
	.insert(TransformBundle::from_transform(Transform::from_translation(PLAYER_SPAWN_POINT)))
	.insert(CameraConfig {
		height_offset: -0.8,
		radius_scale: 0.75,
	})
	.insert(Name::new("Player Entity"))
	.id()
}

pub fn player_drill(
	player_entity	: Entity,
	game_assets		: &GameAssets,
	commands		: &mut Commands,
) -> Entity {
	let player_drill_transform = Transform {
		translation: Vec3::new(0.15, 0.0, -0.6),
		rotation: Quat::from_euler(EulerRot::XYZ, 0.02, 0.2, -1.2),
		..default()
	};

	game_spawn::drill(&player_drill_transform, Some(player_entity), game_assets, commands)
}

pub fn crosshair(
	meshes		: &mut Assets<Mesh>,
	materials	: &mut Assets<StandardMaterial>,
	commands	: &mut Commands,
) -> Entity {
	let crosshair_mesh = meshes.add(Rectangle::new(2.0, 7.0));
	let crosshair_material = materials.add(StandardMaterial {
		base_color : Color::WHITE,
		unlit : true,
		..default()
	});

	let crosshair_entity = commands.spawn((
		Name::new("Crosshair"),
		SpatialBundle {
			transform : Transform {
				translation : Vec3::Z * -1.0,
				..default()
			},
			..default()
		},
		RenderLayers::layer(1),
	)).id();

	let crosshair_part1 = commands.spawn((
		PbrBundle {
			mesh		: crosshair_mesh.clone_weak(),
			material	: crosshair_material.clone_weak(),
			transform	: Transform::from_rotation(Quat::from_rotation_z(PI / 4.0)),
			..default()
		},
		RenderLayers::layer(1),
	)).id();

	let crosshair_part2 = commands.spawn((
		PbrBundle {
			mesh		: crosshair_mesh,
			material	: crosshair_material,
			transform	: Transform::from_rotation(Quat::from_rotation_z(-PI / 4.0)),
			..default()
		},
		RenderLayers::layer(1),
	)).id();

	commands.entity(crosshair_entity).push_children(&[crosshair_part1, crosshair_part2]);

	crosshair_entity
}

pub fn player_camera(
	player_entity	: Entity,
	cubemap			: &Cubemap,
	commands 		: &mut Commands
) -> Entity {
	commands.spawn((
		Camera3dBundle {
			camera : Camera { hdr : true, is_active : true, order: 0, ..default() },
			tonemapping : Tonemapping::TonyMcMapface,
			color_grading: ColorGrading {
				post_saturation: 1.2,
				..default()
			},
			exposure: Exposure { ev100: 6.0 },
			..default()
		},
		RenderPlayer { logical_entity: player_entity },
		Raypick { to_ignore: vec![player_entity], ..default() },
		BloomSettings::NATURAL,
		// Skybox { image : cubemap.image_handle.clone_weak(), brightness : 150.0 },
		EnvironmentMapLight {
			diffuse_map	: cubemap.diffuse_handle.clone_weak(),
			specular_map: cubemap.specular_handle.clone_weak(),
			intensity	: 15.0
		},
		RenderLayers::layer(0),
	)).id()
}

pub fn ui_camera(
	commands : &mut Commands
) -> Entity {
	commands.spawn((
		Camera3dBundle {
			camera : Camera {
				hdr : true,
				is_active : true,
				order: 1,
				clear_color: ClearColorConfig::None,
				..default()
			},
			exposure: Exposure { ev100: 4.0 },
			projection: Projection::Orthographic(Default::default()),

			..default()
		},
		BloomSettings::NATURAL,
		RenderLayers::layer(1),
	)).id()
}

pub fn orbit_camera(
	cubemap		: &Cubemap,
	commands	: &mut Commands
) -> Entity {
	commands.spawn((
		Camera3dBundle {
			camera : Camera { hdr : true, is_active : false, ..default() },
			transform : Transform::from_xyz(0.0, 6.0, 20.0).looking_at(PLAYER_SPAWN_POINT, Vec3::Y),
			tonemapping : Tonemapping::TonyMcMapface,
			color_grading : ColorGrading {
				post_saturation: 1.2,
				..default()
			},
			exposure: Exposure { ev100: 6.0 },
			..default()
		},
		PanOrbitCamera { ..default() },
		BloomSettings::NATURAL,
		// Skybox { image : cubemap.image_handle.clone_weak(), brightness : 150.0 },
		EnvironmentMapLight {
			diffuse_map	: cubemap.diffuse_handle.clone_weak(),
			specular_map: cubemap.specular_handle.clone_weak(),
			intensity	: 15.0
		},
	)).id()
}

pub fn build_menu(
	commands		: &mut Commands
) -> Entity {
	let text_style = TextStyle {
		font_size: 30.0,
		..default()
	};

	commands.spawn((
		TextBundle::from_section("", text_style).with_style(Style {
			position_type: PositionType::Absolute,
			top: Val::Px(150.0),
			right: Val::Px(150.0),
			..default()
		}),
	)).id()
}