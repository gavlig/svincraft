use bevy :: prelude :: *;

use bevy_rapier3d :: prelude :: { RapierPhysicsPlugin, RapierDebugRenderPlugin, NoUserData };
use bevy_fps_controller :: controller :: FpsControllerPlugin;
use bevy_hanabi :: prelude :: HanabiPlugin;
use bevy_panorbit_camera :: PanOrbitCameraPlugin;
use bevy_scene_hook :: HookPlugin;
use bevy_vector_shapes :: prelude :: ShapePlugin;
use bevy_inspector_egui :: quick :: WorldInspectorPlugin;
use iyes_perf_ui :: prelude :: *;

mod assets;
use assets :: AssetsPlugin;

mod setup;
use setup :: SetupPlugin;

mod handheld;
use handheld :: HandheldPlugin;

mod ai;
use ai :: AiPlugin;

mod resource_collection;
use resource_collection :: ResourceCollectionPlugin;

mod game;
use game :: GamePlugin;

mod utils;

fn main() {
	App::new()
		// svincraft plugins
		.add_plugins((
			SetupPlugin,
			AssetsPlugin,
			HandheldPlugin,
			ResourceCollectionPlugin,
			AiPlugin,
			GamePlugin,
		))

		// third party plugins
		.add_plugins((
			RapierPhysicsPlugin::<NoUserData>::default(),
			RapierDebugRenderPlugin { enabled: false, ..default() },
			PanOrbitCameraPlugin,
			FpsControllerPlugin,
			HanabiPlugin,
			HookPlugin,
			ShapePlugin::default(),
			// WorldInspectorPlugin::new(),

			bevy::diagnostic::FrameTimeDiagnosticsPlugin,
			bevy::diagnostic::EntityCountDiagnosticsPlugin,
			bevy::diagnostic::SystemInformationDiagnosticsPlugin,

			// PerfUiPlugin,
		))

		.run();
}
