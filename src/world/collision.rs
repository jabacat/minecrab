use super::generation::World;
use super::blocks::BlockData;

pub struct VoxelRaycastHit {
    //coordinates of the voxel that was hit.
    pub x: i64, pub y: i64, pub z: i64,

    //normal of the face that was hit. could technically be an int since
    //each value will only ever be 0, -1, or 1.
    pub normal_x: f32, pub normal_y: f32, pub normal_z: f32,
    
    //where the hit was on the face.
    pub u: f32, pub v: f32
}

pub fn voxel_raycast(
    world: &World,
    x: f32, y: f32, z: f32,
    dx: f32, dy: f32, dz: f32,
    max_distance: Option<f32>
) -> Option<VoxelRaycastHit> {
    const ROOT2: f32 = 1.4143;
    const MAX_STEPS: i64 = 10_000;
    
    let max_steps = if let Some(md) = max_distance {
        (ROOT2 * md) as i64 + 1
    } else {
        MAX_STEPS
    };

    let direction = [dx, dy, dz];
    let inverse_direction= [ 1. / dx, 1. / dy, 1. / dz ];

    let mut pos = [x, y, z];
    let mut voxel = [x.floor(), y.floor(), z.floor()];

    for _ in 0..max_steps {
        let mut lowest_scale_factor = f32::INFINITY;
        let mut closest_axis = 0usize;


        for axis in 0..3 {
            // block edge that we're "moving toward" on this axis
            let target_coordinate =
                if direction[axis] > 0. { voxel[axis] + 1. }
                else { voxel[axis] };

            let target_dist = target_coordinate - pos[axis];

            // how much do we have to scale the vector by to reach this edge
            let scale_factor = target_dist * inverse_direction[axis];

            if scale_factor > 0. && scale_factor < lowest_scale_factor {
                lowest_scale_factor = scale_factor;
                closest_axis = axis;
            }
        }

        let step = [
            dx * lowest_scale_factor,
            dy * lowest_scale_factor,
            dz * lowest_scale_factor
        ];

        //step position
        for axis in 0..3 { pos[axis] += step[axis]; }

        let step_direction = 
            if direction[closest_axis] > 0. { 1. }
            else { -1. };
        
        //step voxel
        voxel[closest_axis] += step_direction;

        let (vx, vy, vz) = (voxel[0] as i64, voxel[1] as i64, voxel[2] as i64);
        //will be useful for setting hit.u and hit.v
        //let (px, py, pz) = (pos[0] as i64, pos[1] as i64, pos[2] as i64);
        let block = world.get_block_data(vx, vy, vz);

        if block != BlockData::AIR {
            let mut hit = VoxelRaycastHit {
                x: vx, y: vy, z: vz,
                normal_x: 0., normal_y: 0., normal_z: 0.,
                u: 0., v: 0.
            };

            match (closest_axis, step_direction) {

                //TODO: set hit.u and hit.v in here
                (0, -1.) => {
                    hit.normal_x = 1.;
                },
                (0, 1.) => {
                    hit.normal_x = -1.;
                },
                (1, -1.) => {
                    hit.normal_y = 1.;
                },
                (1, 1.) => {
                    hit.normal_y = -1.;
                },
                (2, -1.) => {
                    hit.normal_z = 1.;
                }
                (2, 1.) => {
                    hit.normal_z = -1.;
                }
                _ => panic!("invalid axis/step direction")
            }

            return Some(hit);
        }
    }

    return None;
}
