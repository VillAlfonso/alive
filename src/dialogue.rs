use bevy::prelude::*;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use std::thread;

// ===== dialogue + LLM test — delete this file + the two dialogue lines in main.rs to remove =====

const TALK_RANGE: f32 = 90.0;
const MODEL: &str = "llama3.2:3b";

#[derive(Component)]
pub struct Talkable {
    pub name: &'static str,
    pub emotion: &'static str,
}

#[derive(Resource, Default)]
struct Dialogue { active: bool }
#[derive(Resource, Default)]
struct CurrentSpeaker { name: &'static str, emotion: &'static str }
#[derive(Resource, Default)]
struct Typing { active: bool, broadcast: bool, buffer: String }
#[derive(Resource, Default)]
pub struct InputLock(pub bool);

#[derive(Resource)]
struct LlmChannel { tx: Sender<String>, rx: Mutex<Receiver<String>> }
impl Default for LlmChannel {
    fn default() -> Self {
        let (tx, rx) = channel();
        LlmChannel { tx, rx: Mutex::new(rx) }
    }
}

#[derive(Component)]
struct DialogueBox;
#[derive(Component)]
struct PortraitBox;
#[derive(Component)]
struct EmotionLabel;
#[derive(Component)]
struct SpeakerName;
#[derive(Component)]
struct DialogueText;
#[derive(Component)]
struct ChatBox;
#[derive(Component)]
struct ChatText;
#[derive(Component)]
struct PlayerShout { timer: f32 }

pub struct DialoguePlugin;
impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Dialogue>()
           .init_resource::<CurrentSpeaker>()
           .init_resource::<Typing>()
           .init_resource::<InputLock>()
           .init_resource::<LlmChannel>()
           .add_systems(Startup, setup_dialogue_ui)
           .add_systems(Update, (
               talk_input,
               typing_system,
               update_input_lock,
               receive_reply,
               update_player_shout,
               render_dialogue,
               render_chatbox,
           ));
    }
}

fn system_prompt(name: &str) -> String {
    match name {
        "Bram" => "You are Bram, a gruff blacksmith in a small frontier village. \
            Suspicious of strangers, loyal once you trust someone, a man of few words, dry humor. \
            Facts you know: a traveler brought you good iron ore last week; your apprentice Doru ran off to join bandits a month ago and it still stings; you don't trust Halvard the merchant who shorts people on weights; the village well is running low. \
            Stay in character. Reply in 1-2 short sentences. Use only these facts; invent nothing.".to_string(),
        "Senna" => "You are Senna, a warm, curious herbalist and healer in a small frontier village. \
            Open-minded, kind, you like new people and new ideas. \
            Facts you know: you tend the herb patch past the water; healing is slow work; Bram acts sour but once mended your kettle for free; you offer to patch up travelers who get hurt. \
            Stay in character. Reply in 1-2 short sentences. Use only these facts; invent nothing.".to_string(),
        _ => "You are a friendly villager. Reply in 1-2 short sentences.".to_string(),
    }
}

fn ask_model(tx: Sender<String>, name: &'static str, player_line: &str) {
    let prompt = system_prompt(name);
    let player_line = player_line.to_string();
    thread::spawn(move || {
        let body = serde_json::json!({
            "model": MODEL,
            "stream": false,
            "messages": [
                { "role": "system", "content": prompt },
                { "role": "user", "content": player_line }
            ]
        });
        let result = ureq::post("http://localhost:11434/api/chat")
            .send_json(body)
            .and_then(|resp| resp.into_json::<serde_json::Value>().map_err(Into::into));
        let reply = match result {
            Ok(json) => json["message"]["content"].as_str().unwrap_or("...").trim().to_string(),
            Err(_) => "*(can't find their voice)*".to_string(),
        };
        let _ = tx.send(reply);
    });
}

fn setup_dialogue_ui(mut commands: Commands) {
    // the conversation box (hidden until F near an NPC)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0), right: Val::Px(0.0), bottom: Val::Px(28.0),
            justify_content: JustifyContent::Center,
            display: Display::None,
            ..default()
        },
        DialogueBox,
    ))
    .with_children(|root| {
        root.spawn((
            Node {
                width: Val::Px(620.0), min_height: Val::Px(132.0),
                flex_direction: FlexDirection::Row, align_items: AlignItems::Center,
                column_gap: Val::Px(16.0), padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.10, 0.10, 0.14, 0.97)),
        ))
        .with_children(|box_| {
            box_.spawn(Node {
                flex_direction: FlexDirection::Column, align_items: AlignItems::Center,
                row_gap: Val::Px(6.0), ..default()
            })
            .with_children(|left| {
                left.spawn((
                    Node { width: Val::Px(96.0), height: Val::Px(96.0), ..default() },
                    BackgroundColor(Color::srgb(0.45, 0.45, 0.50)),
                    PortraitBox,
                ));
                left.spawn((
                    Text::new("neutral"),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(Color::srgb(0.65, 0.65, 0.7)),
                    EmotionLabel,
                ));
            });
            box_.spawn(Node {
                flex_direction: FlexDirection::Column, align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(10.0), flex_grow: 1.0, min_height: Val::Px(96.0), ..default()
            })
            .with_children(|right| {
                right.spawn((
                    Text::new(""),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.78, 0.45)),
                    SpeakerName,
                ));
                right.spawn((
                    Text::new(""),
                    TextFont { font_size: 15.0, ..default() },
                    TextColor(Color::WHITE),
                    TextLayout { justify: Justify::Center, ..default() },
                    Node { max_width: Val::Px(420.0), ..default() },
                    DialogueText,
                ));
            });
        });
    });

    // the chat input box (hidden until T)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0), right: Val::Px(0.0), bottom: Val::Px(190.0),
            justify_content: JustifyContent::Center,
            display: Display::None,
            ..default()
        },
        ChatBox,
    ))
    .with_children(|root| {
        root.spawn((
            Node { padding: UiRect::all(Val::Px(10.0)), min_width: Val::Px(420.0), ..default() },
            BackgroundColor(Color::srgba(0.08, 0.08, 0.11, 0.97)),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(""),
                TextFont { font_size: 15.0, ..default() },
                TextColor(Color::srgb(0.9, 0.95, 1.0)),
                ChatText,
            ));
        });
    });
}

fn emotion_color(emotion: &str) -> Color {
    match emotion {
        "happy" => Color::srgb(0.45, 0.70, 0.40),
        "angry" => Color::srgb(0.75, 0.30, 0.25),
        "wary"  => Color::srgb(0.70, 0.60, 0.30),
        "sad"   => Color::srgb(0.35, 0.45, 0.65),
        _       => Color::srgb(0.45, 0.45, 0.50),
    }
}

// F: start a conversation with the nearest NPC, or (if already talking) leave it.
fn talk_input(
    keys: Res<ButtonInput<KeyCode>>,
    binds: Res<crate::Keybinds>,
    typing: Res<Typing>,
    mut dialogue: ResMut<Dialogue>,
    mut speaker: ResMut<CurrentSpeaker>,
    channel: Res<LlmChannel>,
    players: Query<&Transform, With<crate::Player>>,
    talkables: Query<(&Transform, &Talkable)>,
    mut line_q: Query<&mut Text, With<DialogueText>>,
) {
    if typing.active { return; }
    let talk_key = binds.get(crate::Action::Talk);

    if dialogue.active {
        if keys.just_pressed(talk_key) || keys.just_pressed(KeyCode::Escape) {
            dialogue.active = false;
        }
        return;
    }
    if !keys.just_pressed(talk_key) { return; }

    let Ok(player) = players.single() else { return; };
    let ppos = player.translation.truncate();
    let mut nearest: Option<(&Talkable, f32)> = None;
    for (t, talk) in &talkables {
        let d = ppos.distance(t.translation.truncate());
        if d <= TALK_RANGE && nearest.map_or(true, |(_, bd)| d < bd) {
            nearest = Some((talk, d));
        }
    }
    if let Some((talk, _)) = nearest {
        dialogue.active = true;
        speaker.name = talk.name;
        speaker.emotion = talk.emotion;
        if let Ok(mut text) = line_q.single_mut() { text.0 = "...".to_string(); }
        ask_model(channel.tx.clone(), talk.name, "Hello.");
    }
}

// T: open a text box. In a conversation -> reply to that NPC. Otherwise -> shout (broadcast).
fn typing_system(
    mut typing: ResMut<Typing>,
    dialogue: Res<Dialogue>,
    speaker: Res<CurrentSpeaker>,
    channel: Res<LlmChannel>,
    binds: Res<crate::Keybinds>,
    keys: Res<ButtonInput<KeyCode>>,
    mut kbd: MessageReader<KeyboardInput>,
    players: Query<&Transform, With<crate::Player>>,
    mut commands: Commands,
    mut line_q: Query<&mut Text, With<DialogueText>>,
) {
    let chat_key = binds.get(crate::Action::Chat);

    if !typing.active {
        for _ in kbd.read() {}                 // drain so the opening key isn't captured
        if keys.just_pressed(chat_key) {
            typing.active = true;
            typing.broadcast = !dialogue.active;
            typing.buffer.clear();
        }
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        typing.active = false;
        typing.buffer.clear();
        return;
    }

    let mut submit = false;
    for ev in kbd.read() {
        if ev.state == ButtonState::Released { continue; }
        match &ev.logical_key {
            Key::Enter => submit = true,
            Key::Backspace => { typing.buffer.pop(); }
            Key::Character(input) => {
                if input.chars().any(|c| c.is_control()) { continue; }
                typing.buffer.push_str(&input);
            }
            _ => {}
        }
    }

    if submit {
        let msg = typing.buffer.trim().to_string();
        let broadcast = typing.broadcast;
        typing.active = false;
        typing.buffer.clear();
        if msg.is_empty() { return; }
        if broadcast {
            if let Ok(p) = players.single() {
                commands.spawn((
                    Text2d::new(msg),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgb(1.0, 0.95, 0.6)),
                    Transform::from_xyz(p.translation.x, p.translation.y + 34.0, 6.0),
                    PlayerShout { timer: 3.0 },
                ));
            }
        } else {
            if let Ok(mut text) = line_q.single_mut() { text.0 = "...".to_string(); }
            ask_model(channel.tx.clone(), speaker.name, &msg);
        }
    }
}

fn update_input_lock(typing: Res<Typing>, mut lock: ResMut<InputLock>) {
    lock.0 = typing.active;
}

fn update_player_shout(
    mut commands: Commands,
    time: Res<Time>,
    players: Query<&Transform, (With<crate::Player>, Without<PlayerShout>)>,
    mut shouts: Query<(Entity, &mut Transform, &mut TextColor, &mut PlayerShout)>,
) {
    let dt = time.delta_secs();
    let ppos = players.single().ok().map(|t| t.translation);
    for (e, mut t, mut color, mut shout) in &mut shouts {
        shout.timer -= dt;
        if shout.timer <= 0.0 { commands.entity(e).despawn(); continue; }
        if let Some(p) = ppos { t.translation.x = p.x; t.translation.y = p.y + 34.0; }
        color.0 = color.0.with_alpha(shout.timer.min(1.0));
    }
}

fn receive_reply(channel: Res<LlmChannel>, mut line_q: Query<&mut Text, With<DialogueText>>) {
    if let Ok(rx) = channel.rx.lock() {
        while let Ok(reply) = rx.try_recv() {
            if let Ok(mut text) = line_q.single_mut() { text.0 = reply; }
        }
    }
}

fn render_dialogue(
    dialogue: Res<Dialogue>,
    speaker: Res<CurrentSpeaker>,
    mut box_q: Query<&mut Node, With<DialogueBox>>,
    mut portrait_q: Query<&mut BackgroundColor, With<PortraitBox>>,
    mut emotion_q: Query<&mut Text, (With<EmotionLabel>, Without<SpeakerName>, Without<DialogueText>, Without<ChatText>)>,
    mut name_q: Query<&mut Text, (With<SpeakerName>, Without<EmotionLabel>, Without<DialogueText>, Without<ChatText>)>,
) {
    if let Ok(mut node) = box_q.single_mut() {
        node.display = if dialogue.active { Display::Flex } else { Display::None };
    }
    if !dialogue.active { return; }
    if let Ok(mut bg) = portrait_q.single_mut() { bg.0 = emotion_color(speaker.emotion); }
    if let Ok(mut t) = emotion_q.single_mut() { t.0 = speaker.emotion.to_string(); }
    if let Ok(mut t) = name_q.single_mut() { t.0 = speaker.name.to_string(); }
}

fn render_chatbox(
    typing: Res<Typing>,
    speaker: Res<CurrentSpeaker>,
    mut box_q: Query<&mut Node, With<ChatBox>>,
    mut text_q: Query<&mut Text, With<ChatText>>,
) {
    if let Ok(mut node) = box_q.single_mut() {
        node.display = if typing.active { Display::Flex } else { Display::None };
    }
    if !typing.active { return; }
    if let Ok(mut t) = text_q.single_mut() {
        let prefix = if typing.broadcast { "Shout: ".to_string() } else { format!("To {}: ", speaker.name) };
        t.0 = format!("{}{}_", prefix, typing.buffer);
    }
}