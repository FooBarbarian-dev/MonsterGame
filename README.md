# Vegan Monster Cultivation Game

A 2D grid-based creature cultivation game built with Rust and the Bevy ECS framework, featuring non-violent monster capture mechanics and procedurally generated visuals.

## Core Philosophy: Vegan Game Design

This game fundamentally reimagines the creature collector genre by replacing violent combat with ethical, non-violent engagement. The core gameplay loop focuses on:

- **Cultivation over Combat**: Instead of fighting monsters, you help them by addressing their afflictions
- **Ethical Consequences**: The PurityScore system rewards compassionate choices and penalizes aggressive actions
- **Resource-Based Engagement**: Success depends on understanding each creature's needs and applying the right resources

## Key Features

### 🌱 Non-Violent Capture Mechanics

Different cultivar types (plant-based monsters) have unique afflictions that must be addressed:

- **Sprootling**: Suffers from thirst - needs Water
- **Moss-Kneeler**: Afflicted by isolation - requires Tranquilizing Aura
- **Grasping Vine**: Experiences overgrowth - needs specific care
- **Dynamite Bloom**: Unstable - requires puzzle-solving

### 🎮 Core Systems

1. **Grid-Based Movement**: Discrete 2D grid navigation with smooth transitions
2. **State Management**: Clean separation between exploration and encounter states
3. **Procedural Rendering**: All visuals generated using Bevy's mesh primitives (no external sprites)
4. **Purity Score**: Tracks ethical behavior (0-100 scale)
5. **Random Encounters**: 5% chance per step to encounter wild cultivars

## Architecture Highlights

### ECS Components

- **Protagonist**: Player character marker
- **Cultivar**: Plant-based monster entities with unique IDs and archetypes
- **GridCoords**: Integer-based grid positioning
- **PurityScore**: Ethical standing tracker
- **MovementTimer**: Paced movement system

### State System

- **Loading**: Initial setup phase
- **InGame**: Exploration and movement
- **Encounter**: Non-violent interaction with cultivars

### Resources

- **GridScale**: Defines world-to-grid coordinate conversion (64x64 pixels per tile)
- **MapData**: Procedurally generated tile-based world
- **PlayerInventory**: Water, Fertilizer, and Tranquilizing Aura supplies

## Controls

### Exploration Mode
- **Arrow Keys / WASD**: Move on the grid
- Walk on grass to explore and trigger random encounters

### Encounter Mode
- **1**: Use Water (helps Sprootlings with Thirst)
- **2**: Use Fertilizer (for other afflictions)
- **3**: Use Tranquilizing Aura (helps Moss-Kneelers with Isolation)
- **ESC**: Flee encounter (reduces Purity Score by 10 points)

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Basic graphics support (X11 on Linux)

### Building and Running

```bash
# Clone the repository
git clone <repository-url>
cd MonsterGame

# Run the game
cargo run --release

# For development (faster compile, slower runtime)
cargo run
```

### Build Notes

The game uses a minimal Bevy feature set to avoid unnecessary system dependencies:
- Audio disabled (no ALSA requirement)
- Only essential rendering and input features enabled
- Optimized for quick iteration during development

## Technical Implementation

### Procedural Asset Generation

All visual elements are generated at startup using:
- **Meshes**: Circle and Rectangle primitives
- **Materials**: ColorMaterial for simple colored rendering
- **Z-Layering**: Tiles at Z=0, player at Z=10, encounters at Z=15

### Grid Synchronization

The game uses a dual-coordinate system:
1. **GridCoords**: Integer-based logical position
2. **Transform**: Floating-point world position for rendering

The `sync_grid_to_transform` system automatically converts grid positions to world coordinates using the formula:
```
world_x = grid_x * tile_size
world_y = grid_y * tile_size
```

### Encounter System

1. Player movement triggers random encounter check (5% per step)
2. `EncounterTriggerEvent` fired on success
3. State transitions to `AppState::Encounter`
4. Cultivar spawned with random archetype and affliction
5. Player must choose appropriate resource to help the cultivar
6. Success increases Purity Score; fleeing decreases it

## Design Constraints

This game was built under three strict constraints:

1. **Bevy ECS Framework**: Leverages Rust's type system and ECS architecture
2. **No External Sprites**: All visuals procedurally generated using mesh primitives
3. **Vegan Theme**: Core mechanics must reflect non-violent, ethical engagement

These constraints work synergistically:
- Abstract geometric visuals reduce visceral violence associations
- ECS architecture cleanly separates ethical state (PurityScore) from gameplay systems
- Resource-based mechanics replace traditional combat loops

## Future Enhancements

Potential areas for expansion:

- [ ] Dialogue system for Moss-Kneelers (high Purity requirement)
- [ ] Spatial puzzles for Dynamite Bloom encounters
- [ ] Cultivar collection and companion system
- [ ] Multiple biomes with unique cultivar types
- [ ] Advanced purity mechanics (unlocking special abilities)
- [ ] Save/load system for long-term progression

## Code Structure

```
MonsterGame/
├── Cargo.toml          # Dependencies and build configuration
├── src/
│   └── main.rs         # Complete game implementation (~650 lines)
└── README.md           # This file
```

All game logic is contained in a single `main.rs` file organized into:
- State definitions
- ECS components and bundles
- Resources
- System implementations
- Main application setup

## Performance Notes

- Optimized debug builds (`opt-level = 1`)
- Dependency optimization in dev profile
- Release builds use LTO and single codegen unit
- Grid-based visibility optimization ready for large worlds

## License

This project demonstrates architectural patterns for ethical game design using Bevy and Rust.

## Acknowledgments

Built following expert architectural specifications for vegan game design, emphasizing:
- Mechanical enforcement of ethical themes
- ECS-based state management
- Asset-free procedural rendering
- Grid-based spatial optimization

---

**Play compassionately. Build ethically. Cultivate wisely.** 🌱
