use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Inventory>()
        .add_systems(Startup, (setup, setup_hotbar))
        .add_systems(Update, (move_player, follow_player_with_camera).chain())
        .add_systems(Update, (select_hotbar_slot, update_hotbar))
        .run();
}

// ---------- components (tags on entities) ----------
#[derive(Component)]
struct Player;

#[derive(Component)]
struct HotbarSlot { index: usize }   // marks a slot box, remembers which slot it is

#[derive(Component)]
struct SlotLabel { index: usize }    // marks the text inside a slot

// ---------- the inventory data ----------
#[derive(Clone)]
struct ItemStack {
    name: String,
    count: u32,
}

#[derive(Resource)]
struct Inventory {
    slots: Vec<Option<ItemStack>>,   // each slot is either empty (None) or holds a stack
    selected: usize,                 // which hotbar slot is currently active (0..8)
}

impl Default for Inventory {
    fn default() -> Self {
        let mut slots: Vec<Option<ItemStack>> = vec![None; 9];
        slots[0] = Some(ItemStack { name: "Sword".into(), count: 1 });
        slots[1] = Some(ItemStack { name: "Stone".into(), count: 12 });
        slots[2] = Some(ItemStack { name: "Apple".into(), count: 3 });
        Inventory { slots, selected: 0 }
    }
}

// ---------- world setup ----------
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_color(Color::srgb(1.0, 0.48, 0.27), Vec2::new(48.0, 48.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Player,
    ));

    for x in -5..=5 {
        for y in -5..=5 {
            if x == 0 && y == 0 { continue; }
            commands.spawn((
                Sprite::from_color(Color::srgb(0.25, 0.23, 0.20), Vec2::new(32.0, 32.0)),
                Transform::from_xyz(x as f32 * 120.0, y as f32 * 120.0, -1.0),
            ));
        }
    }
}

// ---------- build the hotbar UI ----------
fn setup_hotbar(mut commands: Commands) {
    // a full-screen, see-through container that pins its child to the bottom-centre
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::Center,   // centre horizontally
        align_items: AlignItems::FlexEnd,           // push to the bottom
        padding: UiRect { bottom: Val::Px(16.0), ..default() },
        ..default()
    })
    .with_children(|root| {
        // the dark bar that sits behind the slots
        root.spawn((
            Node {
                flex_direction: FlexDirection::Row,   // lay the slots out left-to-right
                column_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
        ))
        .with_children(|bar| {
            for i in 0..9 {
                bar.spawn((
                    Node {
                        width: Val::Px(50.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.25, 0.25, 0.28, 0.85)),
                    HotbarSlot { index: i },
                ))
                .with_children(|slot| {
                    slot.spawn((
                        Text::new(""),
                        TextFont { font_size: 11.0, ..default() },
                        TextColor(Color::WHITE),
                        TextLayout { justify: Justify::Center, ..default() },
                        SlotLabel { index: i },
                    ));
                });
            }
        });
    });
}

// ---------- hotbar logic ----------
fn select_hotbar_slot(keys: Res<ButtonInput<KeyCode>>, mut inv: ResMut<Inventory>) {
    let digits = [
        KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3,
        KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6,
        KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9,
    ];
    for (i, key) in digits.iter().enumerate() {
        if keys.just_pressed(*key) {
            inv.selected = i;
        }
    }
}

fn update_hotbar(
    inv: Res<Inventory>,
    mut slots: Query<(&HotbarSlot, &mut BackgroundColor)>,
    mut labels: Query<(&SlotLabel, &mut Text)>,
) {
    // repaint each slot: gold if it's the selected one, grey otherwise
    for (slot, mut bg) in &mut slots {
        bg.0 = if slot.index == inv.selected {
            Color::srgba(0.95, 0.78, 0.30, 0.95)
        } else {
            Color::srgba(0.25, 0.25, 0.28, 0.85)
        };
    }
    // write each slot's contents into its label
    for (label, mut text) in &mut labels {
        text.0 = match &inv.slots[label.index] {
            Some(stack) if stack.count > 1 => format!("{}\n{}", stack.name, stack.count),
            Some(stack) => stack.name.clone(),
            None => String::new(),
        };
    }
}

// ---------- movement & camera (unchanged) ----------
fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut players: Query<&mut Transform, With<Player>>,
) {
    let speed = 300.0;
    let dt = time.delta_secs();
    for mut transform in &mut players {
        if keys.pressed(KeyCode::KeyW) { transform.translation.y += speed * dt; }
        if keys.pressed(KeyCode::KeyS) { transform.translation.y -= speed * dt; }
        if keys.pressed(KeyCode::KeyA) { transform.translation.x -= speed * dt; }
        if keys.pressed(KeyCode::KeyD) { transform.translation.x += speed * dt; }
    }
}

fn follow_player_with_camera(
    time: Res<Time>,
    players: Query<&Transform, With<Player>>,
    mut cameras: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok(player) = players.single() else { return; };
    let Ok(mut camera) = cameras.single_mut() else { return; };
    let lerp_speed = 5.0;
    let dt = time.delta_secs();
    camera.translation.x = camera.translation.x.lerp(player.translation.x, lerp_speed * dt);
    camera.translation.y = camera.translation.y.lerp(player.translation.y, lerp_speed * dt);
}