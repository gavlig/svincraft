use bevy :: {
	prelude :: *,
	render :: {
		camera :: CameraProjection,
		primitives :: Frustum,
	},
};

pub fn calc_frustum(
	camera_projection	: &OrthographicProjection,
) -> Frustum {
	let projection_matrix = camera_projection.get_projection_matrix();
	Frustum::from_view_projection_custom_far(
		&projection_matrix,
		&Vec3::ZERO,
		&Vec3::Z,
		camera_projection.far,
	)
}

// in bevy frustum:
// plane 0 is left
// plane 1 is right
// plane 2 is bottom
// plane 3 is top
// plane 4 is near (or back)
// plane 5 is far

pub enum FrustumPlane {
	Left	= 0,
	Right	= 1,
	Bottom	= 2,
	Top		= 3,
//	Near	= 4,
//	Far		= 5
}

// plane equation by three vertices
// Ax + By + Cz + D = 0

pub fn calc_frustum_x_border(frustum: &Frustum, z_in: f32, right: bool) -> f32 {
	let z = z_in;

	let plane_index = if right { FrustumPlane::Right } else { FrustumPlane::Left } as usize;

	let plane = &frustum.half_spaces[plane_index].normal_d();

	(-plane.w - plane.z * z) / plane.x // assume y = 0
}

pub fn calc_frustum_y_border(frustum: &Frustum, z_in: f32, top: bool) -> f32 {
	let z = z_in;

	let plane_index = if top { FrustumPlane::Top } else { FrustumPlane::Bottom } as usize;

	let plane = &frustum.half_spaces[plane_index].normal_d();

	(-plane.w - plane.z * z) / plane.y // assume x = 0
}

// https://x.com/obelexobendre/status/1757879298319323247
pub fn dt_independent_lerp_vec3(from: Vec3, to: Vec3, precision: f32, delta_time: f32, total_time: f32) -> Vec3 {
	let lerp_coef = 1.0 - precision.powf(delta_time / total_time);
	from.lerp(to, lerp_coef)
}

pub fn dt_independent_lerp_f32(from: f32, to: f32, precision: f32, delta_time: f32, total_time: f32) -> f32 {
	let lerp_coef = 1.0 - precision.powf(delta_time / total_time);
	from.lerp(to, lerp_coef)
}

pub fn get_top_ancestor(
	init_entity	: Entity,
	q_parent	: &Query<&Parent>,
) -> Entity {
	let mut root_ancestor = init_entity;
	loop {
		if let Ok(parent_entity) = q_parent.get(root_ancestor) {
			root_ancestor = parent_entity.get();
		} else {
			break;
		}
	}

	root_ancestor
}

pub fn calc_rotation_facing_camera(target_transform: &GlobalTransform, camera_transform: &GlobalTransform) -> Quat {
	let mut to1 = camera_transform.translation();
	to1.y = 0.0;

	let mut to2 = target_transform.translation();
	to2.y = 0.0;

	let arc2 = (to1 - to2).normalize();
	let arc1 = Vec3::Z;

	Quat::from_rotation_arc(arc1, arc2)
}
