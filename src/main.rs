use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Inventory>()
        .init_resource::<InventoryOpen>()
        .add_systems(Startup, (setup, setup_hotbar, setup_inventory))
        .add_systems(Update, (move_player, follow_player_with_camera).chain())
        .add_systems(Update, (
            select_hotbar_slot,
            toggle_inventory,
            apply_inventory_visibility,
            update_slots,
        ))
        .run();
}

// ---------- components (tags on entities) ----------
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Slot { index: usize }          // any item slot (hotbar OR inventory)

#[derive(Component)]
struct SlotLabel { index: usize }     // the text inside a slot

#[derive(Component)]
struct InventoryPanel;                // the whole pop-up inventory screen

// ---------- inventory data ----------
#[derive(Clone)]
struct ItemStack {
    name: String,
    count: u32,
}

#[derive(Resource)]
struct Inventory {
    slots: Vec<Option<ItemStack>>,    // 36 slots: 0..9 = hotbar, 9..36 = main inventory
    selected: usize,                  // active hotbar slot (0..8)
}

impl Default for Inventory {
    fn default() -> Self {
        let mut slots: Vec<Option<ItemStack>> = vec![None; 36];
        slots[0] = Some(ItemStack { name: "Sword".into(), count: 1 });
        slots[1] = Some(ItemStack { name: "Stone".into(), count: 12 });
        slots[2] = Some(ItemStack { name: "Apple".into(), count: 3 });
        slots[9] = Some(ItemStack { name: "Wood".into(), count: 30 });
        slots[10] = Some(ItemStack { name: "Iron".into(), count: 5 });
        Inventory { slots, selected: 0 }
    }
}

#[derive(Resource, Default)]
struct InventoryOpen(bool);           // is the inventory screen showing?

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

// a small helper so we don't repeat the slot-building code
fn spawn_slot(parent: &mut ChildSpawnerCommands, index: usize) {
    parent.spawn((
        Node {
            width: Val::Px(50.0),
            height: Val::Px(50.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.25, 0.25, 0.28, 0.85)),
        Slot { index },
    ))
    .with_children(|slot| {
        slot.spawn((
            Text::new(""),
            TextFont { font_size: 11.0, ..default() },
            TextColor(Color::WHITE),
            TextLayout { justify: Justify::Center, ..default() },
            SlotLabel { index },
        ));
    });
}

// ---------- the hotbar (always visible) ----------
fn setup_hotbar(mut commands: Commands) {
    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::FlexEnd,
        padding: UiRect { bottom: Val::Px(16.0), ..default() },
        ..default()
    })
    .with_children(|root| {
        root.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
        ))
        .with_children(|bar| {
            for i in 0..9 {
                spawn_slot(bar, i);
            }
        });
    });
}

// ---------- the inventory screen (hidden until E) ----------
fn setup_inventory(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            display: Display::None,            // start hidden
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),   // dims the world
        InventoryPanel,
    ))
    .with_children(|backdrop| {
        backdrop.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(14.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.12, 0.15, 0.98)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("Inventory"),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::WHITE),
            ));
            for r in 0..3 {
                panel.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|row| {
                    for c in 0..9 {
                        spawn_slot(row, 9 + r * 9 + c);   // main-inventory indices 9..36
                    }
                });
            }
        });
    });
}

// ---------- logic ----------
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

fn toggle_inventory(keys: Res<ButtonInput<KeyCode>>, mut open: ResMut<InventoryOpen>) {
    if keys.just_pressed(KeyCode::KeyE) {
        open.0 = !open.0;
    }
}

fn apply_inventory_visibility(
    open: Res<InventoryOpen>,
    mut panel: Query<&mut Node, With<InventoryPanel>>,
) {
    if !open.is_changed() { return; }   // only act the moment it toggles
    for mut node in &mut panel {
        node.display = if open.0 { Display::Flex } else { Display::None };
    }
}

fn update_slots(
    inv: Res<Inventory>,
    mut slots: Query<(&Slot, &mut BackgroundColor)>,
    mut labels: Query<(&SlotLabel, &mut Text)>,
) {
    for (slot, mut bg) in &mut slots {
        bg.0 = if slot.index == inv.selected {
            Color::srgba(0.95, 0.78, 0.30, 0.95)   // gold = selected hotbar slot
        } else {
            Color::srgba(0.25, 0.25, 0.28, 0.85)
        };
    }
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