use bevy :: {
	prelude :: *,
	pbr :: DirectionalLightShadowMap,
	window :: { Cursor, CursorGrabMode, PresentMode },
};

use bevy_rapier3d :: prelude :: *;
use polyanya :: Triangulation;
use bevy_hanabi :: prelude :: *;
use iyes_perf_ui :: prelude :: *;

use super :: game :: {
	GameState,
	MainEntities,
};

use super :: assets :: {
	Cubemap,
	GameAssets,
};

use super :: ai :: PolyanyaResource;

use crate :: game :: spawn as game_spawn;

use crate :: handheld :: HandheldOwner;

mod spawn;
use spawn as setup_spawn;

pub struct SetupPlugin;

impl Plugin for SetupPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_plugins(
				DefaultPlugins.set(WindowPlugin {
					primary_window: Some(Window {
						mode: bevy::window::WindowMode::Fullscreen,
						resolution: (1280.0, 800.0).into(),
						cursor: Cursor {
							visible: false,
							grab_mode: CursorGrabMode::Locked,
							..default()
						},
						present_mode: PresentMode::AutoVsync,
						..default()
					}),
					..default()
				})
			)

			.add_systems(OnEnter(GameState::Loaded), (
				setup,
			))

			.insert_resource(RapierConfiguration::default())
			.insert_resource(Msaa::Off)
			.insert_resource(DirectionalLightShadowMap { size: 4096 })
			.insert_resource(ClearColor(Color::BLACK))
			.insert_resource(AmbientLight { brightness: 0.0, ..default() })
		;
	}
}


pub const GROUND_SIZE : f32 = 1000.0;
pub const GROUND_HSIZE : f32 = GROUND_SIZE / 2.0;

fn setup(
		cubemap		: Res<Cubemap>,
	mut game_state	: ResMut<NextState<GameState>>,
	mut game_assets	: ResMut<GameAssets>,
	mut effects		: ResMut<Assets<EffectAsset>>,
	mut meshes		: ResMut<Assets<Mesh>>,
	mut materials	: ResMut<Assets<StandardMaterial>>,
	mut commands	: Commands,
) {
	setup_spawn::light(&mut commands);

	setup_spawn::ground_plane(&mut meshes, &mut materials, &mut commands);

	game_spawn::base_building(Transform::IDENTITY, &game_assets, &mut commands);

	game_spawn::tealite(Transform::from_xyz(3.0, 0.0, 15.0), &game_assets, &mut commands);
	game_spawn::purplite(Transform::from_xyz(-3.0, 0.0, 15.0), &game_assets, &mut commands);

	let resource_ui_entities = setup_spawn::resource_ui(&game_assets, &mut commands);


	let player_entity = setup_spawn::player_entity(&mut commands);

	let drill_entity = setup_spawn::player_drill(player_entity, &game_assets, &mut commands);

	commands.entity(player_entity).insert(HandheldOwner { handheld_entity: drill_entity });

	let crosshair_entity = setup_spawn::crosshair(&mut meshes, &mut materials, &mut commands);

	let build_menu_entity = setup_spawn::build_menu(&mut commands);

	// cameras
	let player_camera_entity = setup_spawn::player_camera(player_entity, &cubemap, &mut commands);

	let ui_camera_entity = setup_spawn::ui_camera(&mut commands);

	let orbit_camera_entity = setup_spawn::orbit_camera(&cubemap, &mut commands);

	commands.entity(player_camera_entity).push_children(&[
		drill_entity,
	]);

	commands.entity(ui_camera_entity).push_children(&[
		resource_ui_entities.purplite,
		resource_ui_entities.tealite,
		crosshair_entity
	]);

	// basic navmesh for plane
	let polyanya_triangulation = Triangulation::from_outer_edges(&[
		Vec2::new(-GROUND_HSIZE, -GROUND_HSIZE),
		Vec2::new(-GROUND_HSIZE,  GROUND_HSIZE),
		Vec2::new( GROUND_HSIZE,  GROUND_HSIZE),
		Vec2::new( GROUND_HSIZE, -GROUND_HSIZE),
	]);

	let Some(mut navmesh) = polyanya_triangulation.as_navmesh() else { panic!("navmesh building failed!") };

	navmesh.bake();

	commands.insert_resource(PolyanyaResource { mesh: navmesh });

	// particle effects
	let resource_drilling_effect = create_resource_drilling_effect(&mut effects);

	let default_drilling_effect = create_default_drilling_effect(&mut effects);

	game_assets.default_drilling_effect = default_drilling_effect;
	game_assets.resource_drilling_effect = resource_drilling_effect;

	// perf ui
	commands.spawn((
	    PerfUiRoot { position : PerfUiPosition::BottomLeft, ..default() },
	    PerfUiEntryFPS::default(),
	    PerfUiEntryFPSWorst::default(),
	    PerfUiEntryFrameTime::default(),
	    PerfUiEntryFrameTimeWorst::default(),
	    PerfUiEntryFrameCount::default(),
	    PerfUiEntryEntityCount::default(),
	    PerfUiEntryCpuUsage::default(),
	    PerfUiEntryMemUsage::default(),
	));

	// inserting resources
	commands.insert_resource(resource_ui_entities);

	commands.insert_resource(MainEntities {
		player			: player_entity,
		player_handheld	: drill_entity,
		player_camera	: player_camera_entity,
		ui_camera		: ui_camera_entity,
		orbit_camera	: orbit_camera_entity,
		build_menu		: build_menu_entity,
	});

	// indicating that we're done with setup system and game is ready to run
	game_state.set(GameState::Main);
}

fn create_resource_drilling_effect(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
	let mut size_gradient = Gradient::new();
	size_gradient.add_key(0.0, Vec2::new(0.01, 0.002));
	size_gradient.add_key(0.5, Vec2::new(0.02, 0.002));
	size_gradient.add_key(0.8, Vec2::new(0.025, 0.003));
	size_gradient.add_key(1.0, Vec2::splat(0.0));

	let spawner = Spawner::rate(500.0.into()).with_starts_active(true);

	let writer = ExprWriter::new();

	let age = writer.lit(0.).expr();
	let init_age = SetAttributeModifier::new(Attribute::AGE, age);

	let lifetime = writer.lit(1.5).expr();
	let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

	let color = writer.prop("color").expr();
	let init_color = SetAttributeModifier::new(Attribute::COLOR, color);

	// Add constant downward acceleration to simulate gravity
	let accel = writer.lit(Vec3::Y * -1.).expr();
	let update_accel = AccelModifier::new(accel);

	let normal = writer.prop("normal");
	let tangent = writer.prop("tangent");

	// Set the position to be the collision point
	let pos = writer.lit(Vec3::ZERO);
	let init_pos = SetAttributeModifier::new(Attribute::POSITION, pos.expr());

	let spread = writer.rand(ScalarType::Float) * writer.lit(10.) - writer.lit(5.);
	let speed = writer.rand(ScalarType::Float) * writer.lit(1.0);
	let velocity = (normal + tangent * spread).normalized() * speed;

	let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, velocity.expr());

	effects.add(EffectAsset::new(32768, spawner, writer.finish())
		.with_name("Sparks")
		.with_property("normal", Vec3::ZERO.into())
		.with_property("tangent", Vec3::ZERO.into())
		.with_property("color", 0xFFFFFFFFu32.into())
		.init(init_pos)
		.init(init_vel)
		.init(init_age)
		.init(init_lifetime)
		.init(init_color)
		.update(update_accel)
		.render(SizeOverLifetimeModifier {
			gradient: size_gradient,
			screen_space_size: false,
		})
		.render(OrientModifier::new(OrientMode::AlongVelocity)),
	)
}

fn create_default_drilling_effect(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
	let mut color_gradient = Gradient::new();
	color_gradient.add_key(0.0, Vec4::splat(1.0));
	color_gradient.add_key(0.1, Vec4::new(1.0, 1.0, 0.0, 1.0));
	color_gradient.add_key(0.4, Vec4::new(1.0, 1.0, 0.0, 1.0));
	color_gradient.add_key(1.0, Vec4::splat(0.0));

	let mut size_gradient = Gradient::new();
	size_gradient.add_key(0.0, Vec2::new(0.01, 0.002));
	size_gradient.add_key(0.5, Vec2::new(0.02, 0.002));
	size_gradient.add_key(0.8, Vec2::new(0.03, 0.003));
	size_gradient.add_key(1.0, Vec2::splat(0.0));

	let spawner = Spawner::rate(100.0.into()).with_starts_active(true);

	let writer = ExprWriter::new();

	let age = writer.lit(0.).expr();
	let init_age = SetAttributeModifier::new(Attribute::AGE, age);

	let lifetime = writer.lit(1.0).expr();
	let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

	// Add constant downward acceleration to simulate gravity
	let accel = writer.lit(Vec3::Y * -1.).expr();
	let update_accel = AccelModifier::new(accel);

	let normal = writer.prop("normal");
	let tangent = writer.prop("tangent");

	// Set the position to be the collision point
	let pos = writer.lit(Vec3::ZERO);
	let init_pos = SetAttributeModifier::new(Attribute::POSITION, pos.expr());

	let spread = writer.rand(ScalarType::Float) * writer.lit(7.) - writer.lit(3.5);
	let speed = writer.rand(ScalarType::Float) * writer.lit(0.5);
	let velocity = (normal + tangent * spread).normalized() * speed;

	let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, velocity.expr());

	effects.add(EffectAsset::new(32768, spawner, writer.finish())
		.with_name("Default Sparks")
		// .with_property("accel", Vec3::new(0., -3., 0.).into())
		.with_property("normal", Vec3::ZERO.into())
		.with_property("tangent", Vec3::ZERO.into())
		.init(init_pos)
		.init(init_vel)
		.init(init_age)
		.init(init_lifetime)
		.update(update_accel)
		.render(ColorOverLifetimeModifier {
			gradient: color_gradient,
		})
		.render(SizeOverLifetimeModifier {
			gradient: size_gradient,
			screen_space_size: false,
		})
		.render(OrientModifier::new(OrientMode::AlongVelocity)),
	)
}