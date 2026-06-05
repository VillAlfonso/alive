use bevy::prelude::*;

// ===== NPC simulation test — delete this file + the two npc lines in main.rs to remove =====

const HUNGER_RATE: f32 = 6.0;
const THIRST_RATE: f32 = 9.0;
const NEED_THRESHOLD: f32 = 60.0;
const NPC_SPEED: f32 = 140.0;
const REACH_DISTANCE: f32 = 30.0;

const WANDER_SPEED: f32 = 55.0;
const WANDER_RADIUS: f32 = 200.0;
const WANDER_PAUSE: f32 = 1.5;

const SOCIAL_RANGE: f32 = 280.0;     // a content friend within this -> go hang out
const SOCIAL_DISTANCE: f32 = 46.0;   // how close they stand

#[derive(Component)]
struct Npc;
#[derive(Component)]
struct NpcId(u32);
#[derive(Component)]
struct Needs { hunger: f32, thirst: f32 }
#[derive(Component)]
struct Home(Vec2);
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
    let villagers = [
        (0u32, Vec2::new(-150.0, -100.0), 0.0, 0.0),
        (1u32, Vec2::new(120.0, -40.0), 10.0, 25.0),
    ];
    for (id, home, hunger, thirst) in villagers {
        commands.spawn((
            Sprite::from_color(Color::srgb(0.3, 0.8, 0.4), Vec2::new(22.0, 22.0)),
            Transform::from_xyz(home.x, home.y, 0.0),
            Npc,
            NpcId(id),
            Needs { hunger, thirst },
            Home(home),
            Wander { target: home, pause: 0.0 },
        ));
    }
    for (id, home, hunger, thirst) in villagers {
        let mut e = commands.spawn((
            Sprite::from_color(Color::srgb(0.3, 0.8, 0.4), Vec2::new(22.0, 22.0)),
            Transform::from_xyz(home.x, home.y, 0.0),
            Npc,
            NpcId(id),
            Needs { hunger, thirst },
            Home(home),
            Wander { target: home, pause: 0.0 },
        ));
        if id == 0 {
            e.insert(crate::dialogue::Talkable { name: "Bram", emotion: "wary" });
        }
    }
    commands.spawn((
        Sprite::from_color(Color::srgb(0.3, 0.5, 0.95), Vec2::new(28.0, 28.0)),
        Transform::from_xyz(300.0, 170.0, -0.5),
        Source::Food,
    ));
    commands.spawn((
        Sprite::from_color(Color::srgb(0.3, 0.85, 0.9), Vec2::new(28.0, 28.0)),
        Transform::from_xyz(-320.0, 190.0, -0.5),
        Source::Water,
    ));
}

fn npc_needs(time: Res<Time>, mut npcs: Query<&mut Needs, With<Npc>>) {
    let dt = time.delta_secs();
    for mut n in &mut npcs {
        n.hunger = (n.hunger + HUNGER_RATE * dt).min(100.0);
        n.thirst = (n.thirst + THIRST_RATE * dt).min(100.0);
    }
}

fn npc_behavior(
    time: Res<Time>,
    sources: Query<(&Transform, &Source), Without<Npc>>,
    mut npcs: Query<(Entity, &mut Transform, &mut Needs, &mut Sprite, &Home, &mut Wander, &NpcId), With<Npc>>,
) {
    let dt = time.delta_secs();

    // snapshot every NPC's position + whether it's content (needs met)
    let mut others: Vec<(Entity, Vec2, bool)> = Vec::new();
    for (e, t, n, _s, _h, _w, _id) in &npcs {
        let content = n.hunger < NEED_THRESHOLD && n.thirst < NEED_THRESHOLD;
        others.push((e, t.translation.truncate(), content));
    }

    for (e, mut t, mut needs, mut sprite, home, mut wander, id) in &mut npcs {
        let want_water = needs.thirst >= needs.hunger;
        let top_need = if want_water { needs.thirst } else { needs.hunger };

        if top_need >= NEED_THRESHOLD {
            // a need is loud -> go satisfy it
            let kind = if want_water { Source::Water } else { Source::Food };
            sprite.color = if want_water { Color::srgb(0.3, 0.6, 0.95) } else { Color::srgb(0.9, 0.6, 0.2) };

            let mut goal = None;
            for (st, s) in &sources {
                if *s == kind { goal = Some(st.translation.truncate()); }
            }
            let Some(goal) = goal else { continue; };

            let to_goal = goal - t.translation.truncate();
            if to_goal.length() > REACH_DISTANCE {
                let step = to_goal.normalize() * NPC_SPEED * dt;
                t.translation.x += step.x;
                t.translation.y += step.y;
            } else if want_water { needs.thirst = 0.0; } else { needs.hunger = 0.0; }
        } else {
            // content -> look for a content friend to hang out with
            let pos = t.translation.truncate();
            let mut friend = None;
            let mut best = SOCIAL_RANGE;
            for (oe, opos, ocontent) in &others {
                if *oe == e || !*ocontent { continue; }
                let d = pos.distance(*opos);
                if d < best { best = d; friend = Some(*opos); }
            }

            if let Some(fpos) = friend {
                // socialize: approach the friend, then stand near them
                sprite.color = Color::srgb(0.6, 0.85, 0.5);
                let to = fpos - pos;
                if to.length() > SOCIAL_DISTANCE {
                    let step = to.normalize() * WANDER_SPEED * dt;
                    t.translation.x += step.x;
                    t.translation.y += step.y;
                }
            } else {
                // nobody around -> wander near home (phase offset keeps the two out of sync)
                sprite.color = Color::srgb(0.3, 0.8, 0.4);
                let phase = id.0 as f32 * 17.3;
                if wander.pause > 0.0 {
                    wander.pause -= dt;
                } else if pos.distance(wander.target) < REACH_DISTANCE {
                    wander.pause = WANDER_PAUSE;
                    let a = time.elapsed_secs() * 2.3 + phase;
                    let dist = WANDER_RADIUS * (0.4 + 0.6 * (time.elapsed_secs() * 1.7 + phase).sin().abs());
                    wander.target = home.0 + Vec2::new(a.cos() * dist, a.sin() * dist);
                } else {
                    let step = (wander.target - pos).normalize() * WANDER_SPEED * dt;
                    t.translation.x += step.x;
                    t.translation.y += step.y;
                }
            }
        }
    }
}