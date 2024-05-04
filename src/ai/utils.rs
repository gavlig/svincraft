use bevy :: {
	prelude :: *,
	render :: {
		mesh :: Indices,
		render_resource :: PrimitiveTopology,
		render_asset :: RenderAssetUsages,
	},
	utils :: tracing,
};

use bevy_vector_shapes :: prelude :: *;

use polyanya :: Mesh as PolyanyaMesh;

use itertools :: Itertools;

use super :: {
	Occupied,
	Locator,
	MovePath,
};

use std :: f32 :: consts :: PI;

// taken from vleue_navigator
pub fn navmesh_to_wireframe(polyanya_mesh: &PolyanyaMesh) -> Mesh {
	let mut new_mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::all());
	new_mesh.insert_attribute(
		Mesh::ATTRIBUTE_POSITION,
		polyanya_mesh
			.vertices
			.iter()
			.map(|v| [v.coords.x, 0.0, v.coords.y])
			.collect::<Vec<[f32; 3]>>(),
	);
	new_mesh.insert_attribute(
		Mesh::ATTRIBUTE_NORMAL,
		polyanya_mesh
			.vertices
			.iter()
			.map(|_| [0.0, 1.0, 0.0])
			.collect::<Vec<[f32; 3]>>(),
	);
	new_mesh.insert_indices(Indices::U32(
		polyanya_mesh
			.polygons
			.iter()
			.flat_map(|p| {
				(0..p.vertices.len())
					.map(|i| [p.vertices[i], p.vertices[(i + 1) % p.vertices.len()]])
			})
			.unique_by(|[a, b]| if a < b { (*a, *b) } else { (*b, *a) })
			.flatten()
			.collect(),
	));

	new_mesh
}

pub fn collect_locators_for_container(
	q_children		: &Query<&Children>,
	q_name			: &Query<&Name>,
	locator			: Locator,
	container_entity: Entity,
	commands		: &mut Commands
) {
	let marker = match locator {
		Locator::Spawn => "spawn_locator",
		Locator::Interact => "interact_locator",
	};

	for descendant in q_children.iter_descendants(container_entity) {
		if let Ok(entity_name) = q_name.get(descendant) {
			if entity_name.as_str().contains(marker) {
				commands.entity(descendant).insert(locator);
			}
		}
	}
}

pub fn pick_nearest_locator(
	locator_in			: Locator,
	container_entity	: &Entity,
	movable_transform	: &Transform,
	q_children			: &Query<&Children>,
	q_locator			: &Query<(&Locator, &GlobalTransform, Option<&Occupied>)>,
) -> Option<(Vec3, Quat, Entity)> {
	let _span = tracing::info_span!("pick_nearest_locator").entered();

	let mut output = None;

	let mut nearest_locator : Option<(Entity, GlobalTransform)> = None;
	let mut nearest_unoccupied_dist_sq = f32::MAX;
	let mut nearest_dist_sq = f32::MAX;

	for locator_entity in q_children.iter_descendants(*container_entity) {
		let Ok((locator, locator_transform, occupied)) = q_locator.get(locator_entity) else { continue };

		if *locator != locator_in { continue }

		let dist_sq = locator_transform.translation().distance_squared(movable_transform.translation);

		if dist_sq < nearest_dist_sq {
			nearest_dist_sq = dist_sq;
			nearest_locator = Some((locator_entity, locator_transform.clone()));
		}

		if occupied.is_some() { continue }

		if dist_sq < nearest_unoccupied_dist_sq {
			nearest_unoccupied_dist_sq = dist_sq;
			let (_, rotation, translation) = locator_transform.to_scale_rotation_translation();

			output = Some((
				translation,
				rotation,
				locator_entity,
			));
		}
	}

	if output.is_none() && nearest_locator.is_some() {
		// if all locators are occupied pick nearest
		let locator = nearest_locator.unwrap();

		let (_, rotation, translation) = locator.1.to_scale_rotation_translation();
		output = Some((
			translation,
			rotation,
			locator.0,
		));
	}

	output
}

pub fn make_path_to_nearest_locator(
	locator				: Locator,
	container_entity	: &Entity,
	movable_transform	: &Transform,
	q_children			: &Query<&Children>,
	q_locator			: &Query<(&Locator, &GlobalTransform, Option<&Occupied>)>,
	navmesh				: &PolyanyaMesh,
) -> Option<MovePath> {
	let Some((locator_pos, locator_rot, locator_entity)) = pick_nearest_locator(
		locator,
		container_entity,
		movable_transform,
		q_children,
		q_locator,
	) else {
		return None;
	};

	let npc_pos2	= movable_transform.translation.xz();
	let locator_pos2= locator_pos.xz();

	if !navmesh.point_in_mesh(locator_pos2) {
		// println!("Error: can't create path to {:?}! locator is not on navmesh!", locator_pos2);
		return None
	}

	let Some(path_wlen) = navmesh.path(npc_pos2, locator_pos2) else {
		// println!("Error: can't create path from {:?} to {:?}! navmesh.path returned None!", npc_pos2, locator_pos2);
		return None
	};

	if let Some((first, remaining)) = path_wlen.path.split_first() {
		let mut remaining = remaining.iter().map(|v| { Vec3::new(v.x, 0.0, v.y) }).collect::<Vec<Vec3>>();
		remaining.reverse();

		let current = Vec3::new(first.x, 0.0, first.y);

		Some(
			MovePath::new(current, remaining, Some(locator_rot), Some(locator_entity))
		)
	} else {
		None
	}
}

use interpolation :: *;

pub fn draw_floor_circle(
	circle_radius	: f32,
	vert_move_coef	: f32,
	progress_offset	: f32,
	position		: &Vec3,
	seconds			: f32,
	anim_time		: f32,
	ease_function	: EaseFunction,
	color			: Color,
	painter			: &mut ShapePainter,
) {
	painter.reset			();
	painter.set_translation	(*position);
	painter.set_rotation	(Quat::IDENTITY);
	painter.set_scale		(Vec3::ONE);

	let time_modifier	= ((seconds % anim_time) / anim_time + progress_offset).abs();
	painter.thickness	= 0.1;
	painter.hollow		= true;
	painter.color		= color;
	painter.translate	(Vec3::Y * 0.1);
	painter.translate	(Vec3::Y * time_modifier.calc(ease_function) * vert_move_coef);
	painter.rotate_x	(PI / 2.0);
	painter.circle		(circle_radius);
}
