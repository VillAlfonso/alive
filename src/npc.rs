use bevy::prelude::*;

// ===== NPC simulation test — delete this file + the two npc lines in main.rs to remove =====

const HUNGER_RATE: f32 = 6.0;
const THIRST_RATE: f32 = 9.0;        // thirst climbs faster than hunger
const NEED_THRESHOLD: f32 = 60.0;    // a need this high demands action
const NPC_SPEED: f32 = 140.0;
const REACH_DISTANCE: f32 = 30.0;

const WANDER_SPEED: f32 = 55.0;      // slow amble when content
const WANDER_RADIUS: f32 = 220.0;    // how far it roams from home
const WANDER_PAUSE: f32 = 1.5;       // seconds it rests between strolls

#[derive(Component)]
struct Npc;
#[derive(Component)]
struct Needs { hunger: f32, thirst: f32 }
#[derive(Component)]
struct Home(Vec2);                   // where it wanders around
#[derive(Component)]
struct Wander { target: Vec2, pause: f32 }
#[derive(Component, PartialEq)]
enum Source { Food, Water }

pub struct NpcPlugin;
impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_npc_test)
           .add_systems(Update, (npc_needs, npc_behavior).chain());
    }
}

fn spawn_npc_test(mut commands: Commands) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.3, 0.8, 0.4), Vec2::new(22.0, 22.0)),
        Transform::from_xyz(-150.0, -100.0, 0.0),
        Npc,
        Needs { hunger: 0.0, thirst: 0.0 },
        Home(Vec2::new(-150.0, -100.0)),
        Wander { target: Vec2::new(-150.0, -100.0), pause: 0.0 },
    ));
    // food (blue) and water (cyan)
    commands.spawn((
        Sprite::from_color(Color::srgb(0.3, 0.5, 0.95), Vec2::new(28.0, 28.0)),
        Transform::from_xyz(280.0, 160.0, -0.5),
        Source::Food,
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.3, 0.85, 0.9), Vec2::new(28.0, 28.0)),
        Transform::from_xyz(-300.0, 180.0, -0.5),
        Source::Water,
    ));
}

// needs climb on their own
fn npc_needs(time: Res<Time>, mut npcs: Query<&mut Needs, With<Npc>>) {
    let dt = time.delta_secs();
    for mut n in &mut npcs {
        n.hunger = (n.hunger + HUNGER_RATE * dt).min(100.0);
        n.thirst = (n.thirst + THIRST_RATE * dt).min(100.0);
    }
}

// decide + act: most urgent need wins; if none, wander
fn npc_behavior(
    time: Res<Time>,
    sources: Query<(&Transform, &Source), Without<Npc>>,
    mut npcs: Query<(&mut Transform, &mut Needs, &mut Sprite, &Home, &mut Wander), With<Npc>>,
) {
    let dt = time.delta_secs();
    for (mut t, mut needs, mut sprite, home, mut wander) in &mut npcs {
        // pick the loudest need (if it's past the threshold)
        let want_water = needs.thirst >= needs.hunger;
        let top_need = if want_water { needs.thirst } else { needs.hunger };

        if top_need >= NEED_THRESHOLD {
            let kind = if want_water { Source::Water } else { Source::Food };
            sprite.color = if want_water { Color::srgb(0.3, 0.6, 0.95) }  // blue = thirsty
                           else { Color::srgb(0.9, 0.6, 0.2) };            // orange = hungry

            // find the matching source
            let mut target = None;
            for (st, s) in &sources {
                if *s == kind { target = Some(st.translation.truncate()); }
            }
            let Some(goal) = target else { continue; };

            let to_goal = goal - t.translation.truncate();
            if to_goal.length() > REACH_DISTANCE {
                let step = to_goal.normalize() * NPC_SPEED * dt;
                t.translation.x += step.x;
                t.translation.y += step.y;
            } else if want_water {
                needs.thirst = 0.0;
            } else {
                needs.hunger = 0.0;
            }
        } else {
            // content -> wander around home
            sprite.color = Color::srgb(0.3, 0.8, 0.4);   // green
            let pos = t.translation.truncate();

            if wander.pause > 0.0 {
                wander.pause -= dt;
            } else if pos.distance(wander.target) < REACH_DISTANCE {
                // arrived -> rest, then pick a new spot near home
                wander.pause = WANDER_PAUSE;
                let angle = time.elapsed_secs() * 2.3;   // cheap pseudo-random direction
                let dist = WANDER_RADIUS * (0.4 + 0.6 * (time.elapsed_secs() * 1.7).sin().abs());
                wander.target = home.0 + Vec2::new(angle.cos() * dist, angle.sin() * dist);
            } else {
                let step = (wander.target - pos).normalize() * WANDER_SPEED * dt;
                t.translation.x += step.x;
                t.translation.y += step.y;
            }
        }
    }
}