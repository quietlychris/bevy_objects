use bevy::gltf::{Gltf, GltfMesh};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_flycam::PlayerPlugin;
use bevy_rapier3d::na::Scale;
use bevy_rapier3d::prelude::*;

#[derive(AssetCollection, Debug)]
struct MyAssets {
    #[asset(path = "simple-BodyPad.gltf")]
    object: Handle<Gltf>,
}

#[derive(Component, Default)]
struct Object;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    AssetLoading,
    Next,
}

fn main() {
    App::new()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Next)
                .with_collection::<MyAssets>(),
        )
        .add_state(GameState::AssetLoading)
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        // .add_plugin(PlayerPlugin)
        .add_system_set(SystemSet::on_enter(GameState::Next).with_system(use_my_assets))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .add_system(print_obj_altitude)
        .add_system(animate_light_direction)
        .add_system(bevy::window::close_on_esc)
        .run();
}

fn use_my_assets(
    mut commands: Commands,
    assets: Res<MyAssets>,
    assets_gltf: Res<Assets<Gltf>>,
    assets_scenes: Res<Assets<Scene>>,
    assets_gltf_mesh: Res<Assets<GltfMesh>>,
    mut assets_mesh: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Add a camera so we can see the debug-render.
    let cr = 4.0; // camera ratio
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(1.0 * cr, 0.25 * cr, 0.25 * cr).looking_at(
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Vec3::Y,
        ),
        ..Default::default()
    });

    const HALF_SIZE: f32 = 1.0;
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -10.0 * HALF_SIZE,
                far: 10.0 * HALF_SIZE,
                ..default()
            },
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });

    let rectangle: shape::Box = shape::Box {
        min_x: -1.5,
        max_x: 1.5,
        min_y: -0.1,
        max_y: 0.0,
        min_z: -1.5,
        max_z: 1.5,
    };

    // Physically-based rendering object
    // This is what will show up in our simulator!
    // Yes, it's technically a rectanglular prism, but that would mean a longer variable name ¯\_(ツ)_/¯

    let mesh_handle_rectangle = assets_mesh.add(Mesh::from(rectangle));
    let cube = PbrBundle {
        mesh: mesh_handle_rectangle.clone(),
        material: materials.add(StandardMaterial {
            base_color: Color::hex("cc0000").unwrap(),
            // vary key PBR parameters on a grid of spheres to show the effect
            metallic: 0.1,
            perceptual_roughness: 0.1,
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, -0.1, 0.0),
        ..Default::default()
    };

    let rectangle_mesh = assets_mesh.get(&mesh_handle_rectangle).unwrap();
    let collider_shape = ComputedColliderShape::ConvexDecomposition(VHACDParameters::default());
    let trimesh = ComputedColliderShape::TriMesh;

    commands
        .spawn_bundle(cube)
        .insert(Object)
        .insert(Collider::from_bevy_mesh(rectangle_mesh, &trimesh).unwrap())
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 0., 0.0)));
    /*
    .insert(Friction::coefficient(0.9))
    .insert(Restitution::coefficient(0.))
    */

    let obj = assets_gltf.get(&assets.object).unwrap();
    dbg!(&obj);

    let obj_scale = 2.0;
    let scene = SceneBundle {
        scene: obj.default_scene.clone().unwrap(),
        transform: Transform {
            translation: Vec3 {
                x: 0.0,
                y: 1.5,
                z: 0.0,
            },
            rotation: Quat::from_euler(EulerRot::XYZ, -90.0, 0.0, 0.0),
            scale: Vec3::new(obj_scale, obj_scale, obj_scale),
        },
        ..default()
    };

    let gltf_mesh = assets_gltf_mesh.get(&obj.meshes[0]).unwrap();

    commands
        .spawn_bundle(scene)
        .insert(Object)
        .insert(RigidBody::Dynamic)
        .with_children(|children| {
            let tb = TransformBundle::from_transform(Transform {
                ..Default::default()
            });

            for i in 0..gltf_mesh.primitives.len() {
                let mesh_handle_object = gltf_mesh.primitives[i].mesh.clone();
                let mesh = assets_mesh.get(&mesh_handle_object).unwrap();
                let collider = Collider::from_bevy_mesh(mesh, &trimesh).unwrap();
                children.spawn_bundle(tb.clone()).insert(collider);
            }
        });
    /*
    .insert(Damping {
        linear_damping: 0.5,
        angular_damping: 0.5,
    })
    .insert(GravityScale(1.0))
    .insert(Friction::coefficient(10.0))
    .insert(Restitution::coefficient(0.1))
    .insert(ColliderMassProperties::Density(1000.0));
    */
}

fn print_obj_altitude(positions: Query<&Transform, With<RigidBody>>) {
    for transform in positions.iter() {
        println!("Object altitude: {}", transform.translation.y);
    }
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.seconds_since_startup() as f32 * std::f32::consts::TAU / 10.0,
            -std::f32::consts::FRAC_PI_4,
        );
    }
}
