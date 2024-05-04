use bevy :: {
	prelude :: *,
	asset :: LoadState,
	render :: render_resource :: { TextureViewDescriptor, TextureViewDimension },
};

use super :: *;

pub fn start_assets_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
	let skybox_texture	= asset_server.load("environment/rosendal/rosendal_park_sunset_1k.png");
	let ibl_diffuse		= asset_server.load("environment/rosendal/diffuse_rgb9e5_zstd.ktx2");
	let ibl_specular	= asset_server.load("environment/rosendal/specular_rgb9e5_zstd.ktx2");

	commands.insert_resource(Cubemap {
		is_loaded		: false,
		image_handle	: skybox_texture,
		diffuse_handle	: ibl_diffuse,
		specular_handle	: ibl_specular,
	});

	let tealite				= asset_server.load("models/tealite.glb#Scene0");
	let tealite_shard		= asset_server.load("models/tealite_shard.glb#Scene0");
	let purplite			= asset_server.load("models/purplite.glb#Scene0");
	let purplite_shard		= asset_server.load("models/purplite_shard.glb#Scene0");
	let base_building		= asset_server.load("models/base_building.glb#Scene0");
	let drill_miller_falls	= asset_server.load("models/miller_falls_drill.glb#Scene0");
	let svin				= asset_server.load("models/svin.glb#Scene0");

	commands.insert_resource(GameAssets {
		tealite,
		tealite_shard,
		purplite,
		purplite_shard,
		base_building,
		drill_miller_falls,
		svin,

		..default()
	});

	commands.insert_resource(Animations(vec![
		asset_server.load("models/miller_falls_drill.glb#Animation0"),
	]));
}

pub fn check_assets_loading(
		cubemap			: Res<Cubemap>,
	mut game_state		: ResMut<NextState<GameState>>,
		game_assets		: Res<GameAssets>,
		animations		: Res<Animations>,
		asset_server	: Res<AssetServer>,
) {
	let scene_handles = game_assets.all_scene_handhles();
	for handle in scene_handles.iter() {
		if asset_server.load_state(handle) != LoadState::Loaded { return }
	}

	// we only have one animation currently
	if asset_server.load_state(&animations.0[0]) != LoadState::Loaded { return }

	if !cubemap.is_loaded { return }

	game_state.set(GameState::Loaded);
}

pub fn check_cubemap_loading(
	mut images			: ResMut<Assets<Image>>,
	mut cubemap			: ResMut<Cubemap>,

		asset_server	: Res<AssetServer>,
) {
	if cubemap.is_loaded {
		return
	}

	if asset_server.load_state(&cubemap.image_handle) == LoadState::Loaded {
		let Some(image) = images.get_mut(&cubemap.image_handle) else { return };

		// NOTE: PNGs do not have any metadata that could indicate they contain a cubemap texture,
		// so they appear as one texture. The following code reconfigures the texture as necessary.
		if image.texture_descriptor.array_layer_count() == 1 {
			image.reinterpret_stacked_2d_as_array(image.height() / image.width());
			image.texture_view_descriptor = Some(TextureViewDescriptor {
				dimension: Some(TextureViewDimension::Cube),
				..default()
			});
		}

		cubemap.is_loaded = true;
	}
}
