use bevy::prelude::*;

// ===== NPC simulation test — delete this file + the two npc lines in main.rs to remove =====

const HUNGER_RATE: f32 = 8.0;        // hunger gained per second
const HUNGER_THRESHOLD: f32 = 60.0;  // at this level the NPC goes to eat
const NPC_SPEED: f32 = 140.0;
const EAT_DISTANCE: f32 = 30.0;

#[derive(Component)]
struct Npc;
#[derive(Component)]
struct Needs { hunger: f32 }   // 0 = full, 100 = starving
#[derive(Component)]
struct Food;

pub struct NpcPlugin;
impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_npc_test)
           .add_systems(Update, (npc_hunger, npc_behavior).chain());
    }
}

fn spawn_npc_test(mut commands: Commands) {
    // the hungry villager
    commands.spawn((
        Sprite::from_color(Color::srgb(0.3, 0.8, 0.4), Vec2::new(22.0, 22.0)),
        Transform::from_xyz(-200.0, -120.0, 0.0),
        Npc,
        Needs { hunger: 0.0 },
    ));
    // something to eat
    commands.spawn((
        Sprite::from_color(Color::srgb(0.3, 0.5, 0.95), Vec2::new(28.0, 28.0)),
        Transform::from_xyz(250.0, 150.0, -0.5),
        Food,
    ));
}

fn npc_hunger(time: Res<Time>, mut npcs: Query<&mut Needs, With<Npc>>) {
    let dt = time.delta_secs();
    for mut needs in &mut npcs {
        needs.hunger = (needs.hunger + HUNGER_RATE * dt).min(100.0);
    }
}

fn npc_behavior(
    time: Res<Time>,
    food: Query<&Transform, (With<Food>, Without<Npc>)>,
    mut npcs: Query<(&mut Transform, &mut Needs, &mut Sprite), With<Npc>>,
) {
    let dt = time.delta_secs();
    let Ok(food_pos) = food.single() else { return; };
    for (mut t, mut needs, mut sprite) in &mut npcs {
        if needs.hunger >= HUNGER_THRESHOLD {
            sprite.color = Color::srgb(0.9, 0.6, 0.2);   // orange = seeking food
            let to_food = food_pos.translation.truncate() - t.translation.truncate();
            if to_food.length() > EAT_DISTANCE {
                let step = to_food.normalize() * NPC_SPEED * dt;
                t.translation.x += step.x;
                t.translation.y += step.y;
            } else {
                needs.hunger = 0.0;   // reached food -> eat -> full
            }
        } else {
            sprite.color = Color::srgb(0.3, 0.8, 0.4);   // green = content
        }
    }
}