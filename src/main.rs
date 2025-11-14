use bevy::prelude::*;
use bevy::sprite::{ColorMaterial, MaterialMesh2dBundle, Mesh2dHandle};
use rand::Rng;

// ============================================================================
// STATE MANAGEMENT
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
enum AppState {
    #[default]
    Loading,
    MainMenu,
    InGame,
    Encounter,
}

// ============================================================================
// CORE COMPONENTS
// ============================================================================

/// Marker component for the player character
#[derive(Component)]
struct Protagonist;

/// Component for cultivar (plant-based monster) entities
#[derive(Component)]
struct Cultivar {
    id: u32,
    rank: u8,
    archetype: CultivarArchetype,
}

/// Different types of cultivars based on the taxonomy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CultivarArchetype {
    Sprootling,      // Circle, Thirst
    MossKneeler,     // Rectangle, Isolation
    GraspingVine,    // Quad, Aggression
    DynamiteBloom,   // Pulsing Circle, Instability
}

/// Integer-based grid position component
#[derive(Component, Clone, Copy)]
struct GridCoords {
    x: i32,
    y: i32,
}

impl GridCoords {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Ethical resource tracking player's non-violent behavior
#[derive(Component)]
struct PurityScore {
    score: f32, // Range: 0.0 to 100.0
}

impl PurityScore {
    fn new() -> Self {
        Self { score: 100.0 }
    }

    fn reduce(&mut self, amount: f32) {
        self.score = (self.score - amount).max(0.0);
        println!("⚠️  Purity Score reduced by {:.1}! Current: {:.1}/100.0", amount, self.score);
    }

    fn increase(&mut self, amount: f32) {
        self.score = (self.score + amount).min(100.0);
        println!("✓ Purity Score increased by {:.1}! Current: {:.1}/100.0", amount, self.score);
    }
}

/// Timer for paced grid movement
#[derive(Component)]
struct MovementTimer {
    timer: Timer,
}

impl MovementTimer {
    fn new(duration_secs: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration_secs, TimerMode::Once),
        }
    }
}

/// Marker for when player is in an active encounter
#[derive(Component)]
struct IsEncountering;

/// Map tile types
#[derive(Clone, Copy, PartialEq, Eq)]
enum TileType {
    Grass,
    Water,
    Stone,
}

/// Component for map tiles
#[derive(Component)]
struct MapTile {
    tile_type: TileType,
}

/// Affliction states for cultivars
#[derive(Component)]
enum Affliction {
    Thirst,
    Isolation,
    Overgrowth,
    Instability,
}

/// Resources that can be used for non-violent capture
#[derive(Component)]
enum CaptureResource {
    Water,
    Fertilizer,
    TranquilizingAura,
}

/// Inventory for the player
#[derive(Resource)]
struct PlayerInventory {
    water: u32,
    fertilizer: u32,
    tranquilizing_aura: u32,
}

impl Default for PlayerInventory {
    fn default() -> Self {
        Self {
            water: 5,
            fertilizer: 3,
            tranquilizing_aura: 2,
        }
    }
}

/// Marker for captured cultivars
#[derive(Component)]
struct Captured;

// ============================================================================
// RESOURCES
// ============================================================================

/// Defines the size of one grid square in world units
#[derive(Resource)]
struct GridScale {
    tile_size: f32,
}

impl Default for GridScale {
    fn default() -> Self {
        Self { tile_size: 64.0 }
    }
}

/// Stores the logical map structure
#[derive(Resource)]
struct MapData {
    tiles: Vec<Vec<TileType>>,
    width: usize,
    height: usize,
}

impl MapData {
    fn new(width: usize, height: usize) -> Self {
        let mut tiles = Vec::new();
        let mut rng = rand::thread_rng();

        for y in 0..height {
            let mut row = Vec::new();
            for x in 0..width {
                // Create a simple pattern: borders are stone, random grass/water inside
                let tile = if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
                    TileType::Stone
                } else if rng.gen_bool(0.15) {
                    TileType::Water
                } else {
                    TileType::Grass
                };
                row.push(tile);
            }
            tiles.push(row);
        }

        Self { tiles, width, height }
    }

    fn get_tile(&self, x: i32, y: i32) -> Option<TileType> {
        if x < 0 || y < 0 {
            return None;
        }
        let ux = x as usize;
        let uy = y as usize;
        if uy < self.tiles.len() && ux < self.tiles[uy].len() {
            Some(self.tiles[uy][ux])
        } else {
            None
        }
    }

    fn is_walkable(&self, x: i32, y: i32) -> bool {
        match self.get_tile(x, y) {
            Some(TileType::Grass) => true,
            Some(TileType::Water) | Some(TileType::Stone) => false,
            None => false,
        }
    }
}

/// Handles for procedurally generated meshes
#[derive(Resource)]
struct ProceduralMeshes {
    player_mesh: Handle<Mesh>,
    sprootling_mesh: Handle<Mesh>,
    moss_kneeler_mesh: Handle<Mesh>,
    tile_mesh: Handle<Mesh>,
}

/// Handles for color materials
#[derive(Resource)]
struct ProceduralMaterials {
    player_material: Handle<ColorMaterial>,
    sprootling_material: Handle<ColorMaterial>,
    moss_kneeler_material: Handle<ColorMaterial>,
    grass_material: Handle<ColorMaterial>,
    water_material: Handle<ColorMaterial>,
    stone_material: Handle<ColorMaterial>,
}

/// Event to trigger a random encounter
#[derive(Event)]
struct EncounterTriggerEvent;

/// Current encounter data
#[derive(Resource, Default)]
struct CurrentEncounter {
    cultivar_entity: Option<Entity>,
    cultivar_archetype: Option<CultivarArchetype>,
}

// ============================================================================
// SYSTEMS
// ============================================================================

/// Setup system: runs once during Loading state
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    info!("🌱 Initializing Vegan Monster Cultivation Game...");

    // Spawn camera
    commands.spawn(Camera2dBundle::default());

    // Create procedural meshes
    let player_mesh = meshes.add(Circle::new(24.0));
    let sprootling_mesh = meshes.add(Circle::new(20.0));
    let moss_kneeler_mesh = meshes.add(Rectangle::new(16.0, 32.0));
    let tile_mesh = meshes.add(Rectangle::new(64.0, 64.0));

    // Create color materials
    let player_material = materials.add(Color::srgb(1.0, 0.6, 0.2)); // Orange
    let sprootling_material = materials.add(Color::srgb(0.5, 1.0, 0.3)); // Light green
    let moss_kneeler_material = materials.add(Color::srgb(0.2, 0.7, 0.3)); // Dark green
    let grass_material = materials.add(Color::srgb(0.3, 0.8, 0.2)); // Grass green
    let water_material = materials.add(Color::srgb(0.2, 0.4, 0.9)); // Blue
    let stone_material = materials.add(Color::srgb(0.5, 0.5, 0.5)); // Gray

    // Store mesh and material handles as resources
    commands.insert_resource(ProceduralMeshes {
        player_mesh,
        sprootling_mesh,
        moss_kneeler_mesh,
        tile_mesh,
    });

    commands.insert_resource(ProceduralMaterials {
        player_material,
        sprootling_material,
        moss_kneeler_material,
        grass_material,
        water_material,
        stone_material,
    });

    // Create and insert map data
    let map_data = MapData::new(20, 15);
    commands.insert_resource(map_data);

    // Insert grid scale resource
    commands.insert_resource(GridScale::default());

    // Insert player inventory
    commands.insert_resource(PlayerInventory::default());

    // Insert current encounter tracking
    commands.insert_resource(CurrentEncounter::default());

    info!("✓ Setup complete! Transitioning to InGame state.");

    // Transition to InGame state
    next_state.set(AppState::InGame);
}

/// Spawn the map tiles when entering InGame state
fn spawn_map(
    mut commands: Commands,
    map_data: Res<MapData>,
    grid_scale: Res<GridScale>,
    meshes: Res<ProceduralMeshes>,
    materials: Res<ProceduralMaterials>,
) {
    info!("🗺️  Spawning map...");

    for y in 0..map_data.height {
        for x in 0..map_data.width {
            let tile_type = map_data.tiles[y][x];
            let material = match tile_type {
                TileType::Grass => materials.grass_material.clone(),
                TileType::Water => materials.water_material.clone(),
                TileType::Stone => materials.stone_material.clone(),
            };

            let world_x = x as f32 * grid_scale.tile_size;
            let world_y = y as f32 * grid_scale.tile_size;

            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.tile_mesh.clone().into(),
                    material,
                    transform: Transform::from_xyz(world_x, world_y, 0.0),
                    ..default()
                },
                MapTile { tile_type },
                GridCoords::new(x as i32, y as i32),
            ));
        }
    }

    info!("✓ Map spawned: {}x{} tiles", map_data.width, map_data.height);
}

/// Spawn the player when entering InGame state
fn spawn_player(
    mut commands: Commands,
    meshes: Res<ProceduralMeshes>,
    materials: Res<ProceduralMaterials>,
    grid_scale: Res<GridScale>,
) {
    info!("🧑 Spawning protagonist...");

    // Start player at grid position (5, 5)
    let start_pos = GridCoords::new(5, 5);
    let world_x = start_pos.x as f32 * grid_scale.tile_size;
    let world_y = start_pos.y as f32 * grid_scale.tile_size;

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.player_mesh.clone().into(),
            material: materials.player_material.clone(),
            transform: Transform::from_xyz(world_x, world_y, 10.0), // Z=10 for foreground
            ..default()
        },
        Protagonist,
        start_pos,
        PurityScore::new(),
        MovementTimer::new(0.2),
    ));

    info!("✓ Protagonist spawned at grid ({}, {})", start_pos.x, start_pos.y);
}

/// Synchronize grid coordinates to transform for rendering
fn sync_grid_to_transform(
    grid_scale: Res<GridScale>,
    mut query: Query<(&GridCoords, &mut Transform), Changed<GridCoords>>,
) {
    for (coords, mut transform) in query.iter_mut() {
        let world_x = coords.x as f32 * grid_scale.tile_size;
        let world_y = coords.y as f32 * grid_scale.tile_size;

        // Keep the original Z coordinate for layering
        transform.translation.x = world_x;
        transform.translation.y = world_y;
    }
}

/// Handle discrete grid-based movement for the player
fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    map_data: Res<MapData>,
    mut player_query: Query<(&mut GridCoords, &mut PurityScore, &mut MovementTimer), With<Protagonist>>,
    time: Res<Time>,
) {
    let Ok((mut coords, mut purity, mut movement_timer)) = player_query.get_single_mut() else {
        return;
    };

    // Tick the movement timer
    movement_timer.timer.tick(time.delta());

    // Only allow movement if timer is finished
    if !movement_timer.timer.finished() {
        return;
    }

    let mut new_x = coords.x;
    let mut new_y = coords.y;
    let mut moved = false;

    // Check input
    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        new_y += 1;
        moved = true;
    } else if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        new_y -= 1;
        moved = true;
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyA) {
        new_x -= 1;
        moved = true;
    } else if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyD) {
        new_x += 1;
        moved = true;
    }

    if moved {
        // Validate movement
        if map_data.is_walkable(new_x, new_y) {
            coords.x = new_x;
            coords.y = new_y;
            movement_timer.timer.reset();
            info!("Player moved to ({}, {})", new_x, new_y);
        } else {
            // Attempted to walk into non-walkable tile - small purity penalty
            purity.reduce(0.5);
            warn!("Cannot walk there! (Stepped on water/stone)");
        }
    }
}

/// Random encounter trigger system
fn check_random_encounter(
    player_query: Query<&GridCoords, (With<Protagonist>, Changed<GridCoords>)>,
    mut encounter_events: EventWriter<EncounterTriggerEvent>,
) {
    // Only trigger on player movement
    if player_query.is_empty() {
        return;
    }

    let mut rng = rand::thread_rng();

    // 5% chance per step
    if rng.gen_bool(0.05) {
        info!("🌿 Random encounter triggered!");
        encounter_events.send(EncounterTriggerEvent);
    }
}

/// Listen for encounter events and transition to Encounter state
fn handle_encounter_event(
    mut encounter_events: EventReader<EncounterTriggerEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for _event in encounter_events.read() {
        info!("→ Transitioning to Encounter state");
        next_state.set(AppState::Encounter);
    }
}

/// Spawn a cultivar when entering Encounter state
fn spawn_encounter_cultivar(
    mut commands: Commands,
    meshes: Res<ProceduralMeshes>,
    materials: Res<ProceduralMaterials>,
    mut current_encounter: ResMut<CurrentEncounter>,
) {
    let mut rng = rand::thread_rng();

    // Randomly choose archetype
    let archetype = if rng.gen_bool(0.5) {
        CultivarArchetype::Sprootling
    } else {
        CultivarArchetype::MossKneeler
    };

    let (mesh, material, affliction) = match archetype {
        CultivarArchetype::Sprootling => (
            meshes.sprootling_mesh.clone(),
            materials.sprootling_material.clone(),
            Affliction::Thirst,
        ),
        CultivarArchetype::MossKneeler => (
            meshes.moss_kneeler_mesh.clone(),
            materials.moss_kneeler_material.clone(),
            Affliction::Isolation,
        ),
        _ => (
            meshes.sprootling_mesh.clone(),
            materials.sprootling_material.clone(),
            Affliction::Thirst,
        ),
    };

    let cultivar_id = commands.spawn((
        MaterialMesh2dBundle {
            mesh: mesh.into(),
            material,
            transform: Transform::from_xyz(0.0, 200.0, 15.0), // Offset above player visually
            ..default()
        },
        Cultivar {
            id: rng.gen(),
            rank: 1,
            archetype,
        },
        affliction,
    )).id();

    current_encounter.cultivar_entity = Some(cultivar_id);
    current_encounter.cultivar_archetype = Some(archetype);

    info!("🌱 A wild {:?} appeared!", archetype);
    info!("💧 Resources - Water: press 1 | Fertilizer: press 2 | Tranquilizing Aura: press 3");
    info!("🚫 Press ESC to flee (reduces purity!)");
}

/// Handle interaction during encounter
fn encounter_interaction(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<PlayerInventory>,
    mut purity_query: Query<&mut PurityScore, With<Protagonist>>,
    cultivar_query: Query<(Entity, &Cultivar, &Affliction)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut current_encounter: ResMut<CurrentEncounter>,
) {
    let Ok(mut purity) = purity_query.get_single_mut() else {
        return;
    };

    let Some(cultivar_entity) = current_encounter.cultivar_entity else {
        return;
    };

    let Ok((entity, cultivar, affliction)) = cultivar_query.get(cultivar_entity) else {
        return;
    };

    let mut end_encounter = false;
    let mut success = false;

    // Resource application
    if keyboard.just_pressed(KeyCode::Digit1) {
        // Use water
        if inventory.water > 0 {
            inventory.water -= 1;
            info!("💧 Used Water resource! Remaining: {}", inventory.water);

            // Check if water helps this affliction
            if matches!(affliction, Affliction::Thirst) {
                info!("✓ The {:?} drinks gratefully and joins you!", cultivar.archetype);
                purity.increase(5.0);
                success = true;
                end_encounter = true;
            } else {
                info!("The {:?} ignores the water.", cultivar.archetype);
            }
        } else {
            warn!("No water remaining!");
        }
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        // Use fertilizer
        if inventory.fertilizer > 0 {
            inventory.fertilizer -= 1;
            info!("🌿 Used Fertilizer! Remaining: {}", inventory.fertilizer);

            // Fertilizer helps with overgrowth (not implemented in this encounter)
            info!("The {:?} seems unaffected.", cultivar.archetype);
        } else {
            warn!("No fertilizer remaining!");
        }
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        // Use tranquilizing aura
        if inventory.tranquilizing_aura > 0 {
            inventory.tranquilizing_aura -= 1;
            info!("✨ Used Tranquilizing Aura! Remaining: {}", inventory.tranquilizing_aura);

            // Aura helps with isolation
            if matches!(affliction, Affliction::Isolation) {
                info!("✓ The {:?} feels safe and joins you!", cultivar.archetype);
                purity.increase(5.0);
                success = true;
                end_encounter = true;
            } else {
                info!("The {:?} is already calm.", cultivar.archetype);
            }
        } else {
            warn!("No tranquilizing aura remaining!");
        }
    } else if keyboard.just_pressed(KeyCode::Escape) {
        // Flee - reduces purity
        info!("You fled from the encounter.");
        purity.reduce(10.0);
        end_encounter = true;
    }

    if end_encounter {
        // Mark as captured if successful
        if success {
            commands.entity(entity).insert(Captured);
            info!("🎉 Cultivar captured and added to your collection!");
        } else {
            // Despawn if fled or failed
            commands.entity(entity).despawn_recursive();
        }

        // Clear encounter data
        current_encounter.cultivar_entity = None;
        current_encounter.cultivar_archetype = None;

        // Return to InGame state
        next_state.set(AppState::InGame);
        info!("← Returning to exploration");
    }
}

/// Cleanup encounter state when exiting
fn cleanup_encounter(
    mut commands: Commands,
    cultivar_query: Query<Entity, With<Cultivar>>,
    mut current_encounter: ResMut<CurrentEncounter>,
) {
    // Despawn any remaining cultivars
    for entity in cultivar_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    current_encounter.cultivar_entity = None;
    current_encounter.cultivar_archetype = None;
}

/// Display purity score in console periodically
fn display_purity_score(
    purity_query: Query<&PurityScore, (With<Protagonist>, Changed<PurityScore>)>,
) {
    for purity in purity_query.iter() {
        info!("🌟 Current Purity Score: {:.1}/100.0", purity.score);
    }
}

// ============================================================================
// MAIN APPLICATION
// ============================================================================

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Vegan Monster Cultivation Game".to_string(),
                resolution: (1280.0, 960.0).into(),
                ..default()
            }),
            ..default()
        }))
        // Register state
        .init_state::<AppState>()
        // Register events
        .add_event::<EncounterTriggerEvent>()
        // Startup systems (Loading state)
        .add_systems(Startup, setup)
        // InGame state entry
        .add_systems(OnEnter(AppState::InGame), (spawn_map, spawn_player).chain())
        // InGame state systems
        .add_systems(
            Update,
            (
                player_movement,
                check_random_encounter,
                handle_encounter_event,
                sync_grid_to_transform,
                display_purity_score,
            )
                .run_if(in_state(AppState::InGame)),
        )
        // Encounter state entry
        .add_systems(OnEnter(AppState::Encounter), spawn_encounter_cultivar)
        // Encounter state systems
        .add_systems(
            Update,
            encounter_interaction.run_if(in_state(AppState::Encounter)),
        )
        // Encounter state exit
        .add_systems(OnExit(AppState::Encounter), cleanup_encounter)
        .run();
}
