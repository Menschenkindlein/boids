use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use rand::prelude::*;

#[derive(Clone, Copy)]
struct Boid {
    position: Vec3,
    rotation: Quat,
}

#[derive(Resource)]
struct Boids(Vec<Boid>);

#[derive(Component)]
struct BoidRef(usize);

fn add_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

const BASE_DIRECTION: Vec3 = Vec3::new(0., 1., 0.);
const VELOCITY: f32 = 100.;

fn diff_as_quat(from: Vec3, to: Vec3) -> Quat {
    let rotation_axis = from.cross(to);
    let rotation_angle = from.angle_between(to);
    let q = Quat::from_axis_angle(rotation_axis.normalize(), rotation_angle).normalize();
    if q.is_nan() || q.is_near_identity() {
        if rand::random() {
            Quat::IDENTITY
        } else {
            Quat::from_rotation_z(std::f32::consts::PI)
        }
    } else {
        q
    }
}

fn add_boids(
    mut boids: ResMut<Boids>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut rng = rand::thread_rng();
    for i in 0..300 {
        let pos_x = 500. - rng.gen::<f32>() * 1000.;
        let pos_y = 250. - rng.gen::<f32>() * 500.;
        let vel_x = 1. - rng.gen::<f32>() * 2.;
        let vel_y = 1. - rng.gen::<f32>() * 2.;
        let position = Vec3::new(pos_x, pos_y, 0.);
        let rotation = diff_as_quat(BASE_DIRECTION, Vec3::new(vel_x, vel_y, 0.));
        boids.0.push(Boid { position, rotation });
        commands.spawn((
            BoidRef(i),
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::RegularPolygon::new(5., 3).into()).into(),
                material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
                transform: Transform::from_translation(position).with_rotation(rotation),
                ..default()
            },
        ));
        // commands.spawn(
        //     MaterialMesh2dBundle {
        //         mesh: meshes.add(shape::RegularPolygon::new(5., 8).into()).into(),
        //         material: materials.add(ColorMaterial::from(Color::WHITE)),
        //         transform: Transform::from_translation(Vec3::new(0.,0.,0.)),
        //         ..default()
        //     },
        // );
    }
}

fn move_boids(mut boids: ResMut<Boids>, time: Res<Time>) {
    for boid in &mut boids.0 {
        let velocity = boid.rotation.mul_vec3(BASE_DIRECTION * VELOCITY);
        boid.position.x += velocity.x * time.delta_seconds();
        boid.position.y += velocity.y * time.delta_seconds();
        if boid.position.x.abs() > 500. || boid.position.y.abs() > 250. {
            boid.rotation = boid
                .rotation
                .mul_quat(Quat::from_rotation_z(std::f32::consts::PI));
        }
        if boid.position.y.abs() > 250. {
            boid.position.y *= 0.9;
        }
        if boid.position.x.abs() > 500. {
            boid.position.x *= 0.9;
        }
    }
}
fn draw_boids(boids: Res<Boids>, mut query: Query<(&BoidRef, &mut Transform), With<BoidRef>>) {
    for (br, mut transform) in &mut query {
        let boid = &boids.0[br.0];
        transform.translation = boid.position;
        transform.rotation = boid.rotation;
    }
}

const NEIGHBOR_DISTANCE_SQUARED: f32 = 50.0 * 50.0;
const NEIGHBOR_ANGLE: f32 = 2.79; // 160.0.to_radians();

fn is_neighbor(me: &Boid, other: &Boid) -> bool {
    let direction = other.position - me.position;

    // Check distance criterion
    if direction.length_squared() >= NEIGHBOR_DISTANCE_SQUARED {
        return false;
    }

    // Calculate the angle between boids in degrees
    let angle = me
        .rotation
        .mul_vec3(BASE_DIRECTION)
        .angle_between(direction);

    // Check angle criterion
    angle < NEIGHBOR_ANGLE
}

fn update_direction_of_boids(mut boids: ResMut<Boids>) {
    let mut updates = Vec::new();
    for boid in &boids.0 {
        let mut neighbors = Vec::new();
        for b in &boids.0 {
            if is_neighbor(boid, b) {
                neighbors.push(b);
            }
        }
        if neighbors.is_empty() {
            updates.push((Quat::IDENTITY, boid.rotation));
            continue;
        }
        let vel = boid.rotation.mul_vec3(BASE_DIRECTION);
        let avg_pos = neighbors.iter().map(|b| b.position).sum::<Vec3>() / neighbors.len() as f32;
        // let center = diff_as_quat(vel, Vec3::ZERO - boid.position);
        let convergence = diff_as_quat(vel, avg_pos - boid.position);
        let avoidance = neighbors
            .iter()
            .map(|b| {
                if (boid.position - b.position).length_squared() < NEIGHBOR_DISTANCE_SQUARED / 4. {
                    (boid.position - b.position)
                        / (boid.position - b.position).length_squared().max(1.0)
                } else {
                    Vec3::ZERO
                }
            })
            .sum::<Vec3>();
        let avoidance = diff_as_quat(vel, avoidance);
        let avg_rot = neighbors.iter().map(|b| b.rotation).sum::<Quat>() / neighbors.len() as f32;
        updates.push((
            (//center * 2. +
                convergence * 10. + avoidance * 11.).normalize(),
            avg_rot,
        ));
    }

    for (boid, (upd, avg_rot)) in boids.0.iter_mut().zip(updates) {
        boid.rotation *= upd / 100.;
        boid.rotation = (boid.rotation * 0.95 + avg_rot * 0.05).normalize();
    }
}

struct BoidsPlugin;

impl Plugin for BoidsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Boids(Vec::new()))
            .add_systems(Startup, (add_camera, add_boids))
            .add_systems(Update, (update_direction_of_boids, move_boids))
            .add_systems(FixedUpdate, draw_boids);
    }
}

fn main() {
    App::new().add_plugins((DefaultPlugins, BoidsPlugin)).run();
}
