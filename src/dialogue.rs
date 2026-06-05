use bevy::prelude::*;
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

pub struct DialoguePlugin;
impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Dialogue>()
           .init_resource::<CurrentSpeaker>()
           .init_resource::<LlmChannel>()
           .add_systems(Startup, setup_dialogue_ui)
           .add_systems(Update, (talk_input, receive_reply, render_dialogue).chain());
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

fn talk_input(
    keys: Res<ButtonInput<KeyCode>>,
    binds: Res<crate::Keybinds>,
    mut dialogue: ResMut<Dialogue>,
    mut speaker: ResMut<CurrentSpeaker>,
    channel: Res<LlmChannel>,
    players: Query<&Transform, With<crate::Player>>,
    talkables: Query<(&Transform, &Talkable)>,
    mut line_q: Query<&mut Text, With<DialogueText>>,
) {
    let talk_key = binds.get(crate::Action::Talk);

    if dialogue.active && keys.just_pressed(KeyCode::Escape) {
        dialogue.active = false;
        return;
    }
    if dialogue.active || !keys.just_pressed(talk_key) { return; }

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
    mut emotion_q: Query<&mut Text, (With<EmotionLabel>, Without<SpeakerName>, Without<DialogueText>)>,
    mut name_q: Query<&mut Text, (With<SpeakerName>, Without<EmotionLabel>, Without<DialogueText>)>,
) {
    if let Ok(mut node) = box_q.single_mut() {
        node.display = if dialogue.active { Display::Flex } else { Display::None };
    }
    if !dialogue.active { return; }
    if let Ok(mut bg) = portrait_q.single_mut() { bg.0 = emotion_color(speaker.emotion); }
    if let Ok(mut t) = emotion_q.single_mut() { t.0 = speaker.emotion.to_string(); }
    if let Ok(mut t) = name_q.single_mut() { t.0 = speaker.name.to_string(); }
}