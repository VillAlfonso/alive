use bevy::prelude::*;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

// ===== NPC simulation test — delete this file + the two npc lines in main.rs to remove =====

const HUNGER_RATE: f32 = 6.0;
const THIRST_RATE: f32 = 9.0;
const NEED_THRESHOLD: f32 = 60.0;
const NPC_SPEED: f32 = 140.0;
const REACH_DISTANCE: f32 = 30.0;

const WANDER_SPEED: f32 = 55.0;
const WANDER_RADIUS: f32 = 200.0;
const WANDER_PAUSE: f32 = 1.5;

const SOCIAL_RANGE: f32 = 280.0;
const SOCIAL_DISTANCE: f32 = 46.0;

const BUBBLE_DURATION: f32 = 3.0;
const BUBBLE_HEIGHT: f32 = 34.0;

const HEAR_RANGE: f32 = 420.0;        // how far a shout carries
const RESPOND_THRESHOLD: f32 = 0.35;  // interest needed to bother answering

#[derive(Component)]
struct Npc;
#[derive(Component)]
struct NpcId(u32);
#[derive(Component)]
struct Needs { hunger: f32, thirst: f32 }
#[derive(Component)]
struct Sociability(f32);   // 0..1 — how readily this NPC engages
#[derive(Component)]
struct Home(Vec2);
#[derive(Component)]
struct Wander { target: Vec2, pause: f32 }
#[derive(Component, PartialEq)]
enum Source { Food, Water }
#[derive(Component)]
struct Speech { cooldown: f32 }
#[derive(Component)]
struct SpeechBubble { timer: f32, owner: Entity }

// replies for NPC bubbles come back from the model on this channel
#[derive(Resource)]
struct NpcLlmChannel { tx: Sender<(Entity, String)>, rx: Mutex<Receiver<(Entity, String)>> }
impl Default for NpcLlmChannel {
    fn default() -> Self {
        let (tx, rx) = channel();
        NpcLlmChannel { tx, rx: Mutex::new(rx) }
    }
}

pub struct NpcPlugin;
impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NpcLlmChannel>()
           .add_systems(Startup, spawn_npc_test)
           .add_systems(Update, (
               npc_needs,
               npc_behavior,
               npc_speak,
               update_bubbles,
               npc_hear_broadcast,
               receive_npc_lines,
           ));
    }
}

fn spawn_npc_test(mut commands: Commands) {
    // id, home, hunger, thirst, sociability
    let villagers = [
        (0u32, Vec2::new(-150.0, -100.0), 0.0, 0.0, 0.45),
        (1u32, Vec2::new(120.0, -40.0), 10.0, 25.0, 0.85),
    ];
    for (id, home, hunger, thirst, soc) in villagers {
        let mut e = commands.spawn((
            Sprite::from_color(Color::srgb(0.3, 0.8, 0.4), Vec2::new(22.0, 22.0)),
            Transform::from_xyz(home.x, home.y, 0.0),
            Npc,
            NpcId(id),
            Needs { hunger, thirst },
            Sociability(soc),
            Home(home),
            Wander { target: home, pause: 0.0 },
            Speech { cooldown: 2.0 },
        ));
        match id {
            0 => { e.insert(crate::dialogue::Talkable { name: "Bram", emotion: "wary" }); }
            1 => { e.insert(crate::dialogue::Talkable { name: "Senna", emotion: "happy" }); }
            _ => {}
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

    let mut others: Vec<(Entity, Vec2, bool)> = Vec::new();
    for (e, t, n, _s, _h, _w, _id) in &npcs {
        let content = n.hunger < NEED_THRESHOLD && n.thirst < NEED_THRESHOLD;
        others.push((e, t.translation.truncate(), content));
    }

    for (e, mut t, mut needs, mut sprite, home, mut wander, id) in &mut npcs {
        let want_water = needs.thirst >= needs.hunger;
        let top_need = if want_water { needs.thirst } else { needs.hunger };

        if top_need >= NEED_THRESHOLD {
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
            let pos = t.translation.truncate();
            let mut friend = None;
            let mut best = SOCIAL_RANGE;
            for (oe, opos, ocontent) in &others {
                if *oe == e || !*ocontent { continue; }
                let d = pos.distance(*opos);
                if d < best { best = d; friend = Some(*opos); }
            }

            if let Some(fpos) = friend {
                sprite.color = Color::srgb(0.6, 0.85, 0.5);
                let to = fpos - pos;
                if to.length() > SOCIAL_DISTANCE {
                    let step = to.normalize() * WANDER_SPEED * dt;
                    t.translation.x += step.x;
                    t.translation.y += step.y;
                }
            } else {
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

fn npc_speak(
    mut commands: Commands,
    time: Res<Time>,
    mut npcs: Query<(Entity, &Transform, &Needs, &mut Speech), With<Npc>>,
) {
    let dt = time.delta_secs();
    for (npc, t, needs, mut speech) in &mut npcs {
        speech.cooldown -= dt;
        if speech.cooldown > 0.0 { continue; }
        speech.cooldown = 4.0 + (t.translation.x.abs() % 3.0);

        let (line, color) = if needs.thirst >= NEED_THRESHOLD {
            ("So dry... need water.", Color::srgb(0.55, 0.75, 0.95))
        } else if needs.hunger >= NEED_THRESHOLD {
            ("My stomach's growling.", Color::srgb(0.95, 0.65, 0.35))
        } else {
            ("Fine day, this.", Color::srgb(0.75, 0.85, 0.7))
        };

        let pos = t.translation;
        commands.spawn((
            Text2d::new(line),
            TextFont { font_size: 14.0, ..default() },
            TextColor(color),
            Transform::from_xyz(pos.x, pos.y + BUBBLE_HEIGHT, 5.0),
            SpeechBubble { timer: BUBBLE_DURATION, owner: npc },
        ));
    }
}

fn update_bubbles(
    mut commands: Commands,
    time: Res<Time>,
    npcs: Query<&Transform, (With<Npc>, Without<SpeechBubble>)>,
    mut bubbles: Query<(Entity, &mut Transform, &mut TextColor, &mut SpeechBubble)>,
) {
    let dt = time.delta_secs();
    for (e, mut t, mut color, mut bubble) in &mut bubbles {
        bubble.timer -= dt;
        if bubble.timer <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        if let Ok(owner_t) = npcs.get(bubble.owner) {
            t.translation.x = owner_t.translation.x;
            t.translation.y = owner_t.translation.y + BUBBLE_HEIGHT;
        }
        let alpha = bubble.timer.min(1.0);
        color.0 = color.0.with_alpha(alpha);
    }
}

// a shout went out -> every nearby NPC judges its own interest; the keenest one answers
fn npc_hear_broadcast(
    mut pending: ResMut<crate::dialogue::PendingBroadcast>,
    channel: Res<NpcLlmChannel>,
    npcs: Query<(Entity, &Transform, &Needs, &Sociability, &crate::dialogue::Talkable)>,
) {
    let Some(msg) = pending.0.take() else { return; };   // consume the shout

    let mut best: Option<(Entity, &'static str, f32)> = None;
    for (e, t, needs, soc, talk) in &npcs {
        let dist = t.translation.truncate().distance(msg.pos);
        if dist > HEAR_RANGE { continue; }                       // out of earshot
        let content = needs.hunger < NEED_THRESHOLD && needs.thirst < NEED_THRESHOLD;
        if !content { continue; }                                // too busy/needy to care
        let interest = soc.0 - dist / HEAR_RANGE;                // social, and close = keener
        if interest < RESPOND_THRESHOLD { continue; }
        if best.map_or(true, |(_, _, b)| interest > b) {
            best = Some((e, talk.name, interest));
        }
    }

    if let Some((entity, name, _)) = best {
        let situation = format!(
            "Someone nearby shouts: \"{}\". You overhear it from across the way. \
             React briefly in character — answer them, or react to what they said.",
            msg.text
        );
        crate::dialogue::request_npc_line(channel.tx.clone(), entity, name, situation);
    }
}

// model replies for NPCs arrive here -> float them as a bubble over the right NPC
fn receive_npc_lines(
    mut commands: Commands,
    channel: Res<NpcLlmChannel>,
    npcs: Query<&Transform, With<Npc>>,
) {
    if let Ok(rx) = channel.rx.lock() {
        while let Ok((entity, reply)) = rx.try_recv() {
            if let Ok(t) = npcs.get(entity) {
                commands.spawn((
                    Text2d::new(reply),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(0.85, 0.95, 1.0)),
                    Transform::from_xyz(t.translation.x, t.translation.y + BUBBLE_HEIGHT, 5.0),
                    SpeechBubble { timer: BUBBLE_DURATION + 2.0, owner: entity },
                ));
            }
        }
    }
}