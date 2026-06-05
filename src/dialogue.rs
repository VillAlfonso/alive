use bevy::prelude::*;

// ===== dialogue box test — delete this file + the two dialogue lines in main.rs to remove =====

const TALK_RANGE: f32 = 90.0;

#[derive(Component)]
pub struct Talkable {
    pub name: &'static str,
    pub emotion: &'static str,
}

#[derive(Resource, Default)]
struct Dialogue { active: bool, line_index: usize }
#[derive(Resource, Default)]
struct CurrentSpeaker { name: &'static str, emotion: &'static str }

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
           .add_systems(Startup, setup_dialogue_ui)
           .add_systems(Update, (talk_input, render_dialogue).chain());
    }
}

// ---- THE function you'll later swap for a model call ----
fn placeholder_line(name: &str, line_index: usize) -> Option<String> {
    let lines: &[&str] = match name {
        "Senna" => &[
            "*smiles* You're new around here. I like new.",
            "I tend the herbs past the water. Healing's a slow art.",
            "Bram acts sour, but he mended my kettle for nothing once. Don't tell him I told you.",
            "Come find me if you ever need patching up.",
        ],
        "Bram" => &[
            "*grunts* What brings you to my forge?",
            "That iron you brought was decent. Don't let it go to your head.",
            "Halvard shorts folks on their weights. Watch him, traveler.",
            "I've no time to stand about. Speak, or move along.",
        ],
        _ => &[
            "Oh — hello there.",
            "Quiet day, isn't it?",
            "I should get back to my work.",
        ],
    };
    lines.get(line_index).map(|s| s.to_string())
}

fn emotion_color(emotion: &str) -> Color {
    match emotion {
        "happy"     => Color::srgb(0.45, 0.70, 0.40),
        "angry"     => Color::srgb(0.75, 0.30, 0.25),
        "wary"      => Color::srgb(0.70, 0.60, 0.30),
        "sad"       => Color::srgb(0.35, 0.45, 0.65),
        _           => Color::srgb(0.45, 0.45, 0.50), // neutral
    }
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
                column_gap: Val::Px(16.0),
                padding: UiRect::all(Val::Px(16.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.10, 0.10, 0.14, 0.97)),
        ))
        .with_children(|box_| {
            // ---- left: portrait + emotion label ----
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

            // ---- right: name (top) + the spoken line (centered) ----
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


fn talk_input(
    keys: Res<ButtonInput<KeyCode>>,
    binds: Res<crate::Keybinds>,
    mut dialogue: ResMut<Dialogue>,
    mut speaker: ResMut<CurrentSpeaker>,
    players: Query<&Transform, With<crate::Player>>,
    talkables: Query<(&Transform, &Talkable)>,
) {
    let talk_key = binds.get(crate::Action::Talk);

    if dialogue.active && keys.just_pressed(KeyCode::Escape) {
        dialogue.active = false;
        return;
    }

    if !dialogue.active {
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
            dialogue.line_index = 0;
            speaker.name = talk.name;
            speaker.emotion = talk.emotion;
        }
    } else {
        if keys.just_pressed(talk_key) {
            dialogue.line_index += 1;
            if placeholder_line(speaker.name, dialogue.line_index).is_none() {
                dialogue.active = false;
            }
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
    mut line_q: Query<&mut Text, (With<DialogueText>, Without<EmotionLabel>, Without<SpeakerName>)>,
) {
    if let Ok(mut node) = box_q.single_mut() {
        node.display = if dialogue.active { Display::Flex } else { Display::None };
    }
    if !dialogue.active { return; }

    if let Ok(mut bg) = portrait_q.single_mut() { bg.0 = emotion_color(speaker.emotion); }
    if let Ok(mut t) = emotion_q.single_mut() { t.0 = speaker.emotion.to_string(); }
    if let Ok(mut t) = name_q.single_mut() { t.0 = speaker.name.to_string(); }
    if let Ok(mut t) = line_q.single_mut() {
        t.0 = placeholder_line(speaker.name, dialogue.line_index).unwrap_or_default();
    }
}