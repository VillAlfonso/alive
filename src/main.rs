use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<Inventory>()
        .init_resource::<InventoryOpen>()
        .init_resource::<SettingsOpen>()
        .init_resource::<Keybinds>()
        .init_resource::<Rebinding>()
        .add_systems(Startup, (setup, setup_hotbar, setup_inventory, setup_settings))
        .add_systems(Update, (move_player, follow_player_with_camera).chain())
        .add_systems(Update, (
            select_hotbar_slot,
            toggle_inventory,
            apply_inventory_visibility,
            settings_button,
            apply_settings_visibility,
            rebind_button,
            capture_rebind,
            escape_handler,
            update_rebind_labels,
            update_slots,
        ))
        .run();
}

// ---------- components ----------
#[derive(Component)]
struct Player;
#[derive(Component)]
struct Slot { index: usize }
#[derive(Component)]
struct SlotLabel { index: usize }
#[derive(Component)]
struct InventoryPanel;
#[derive(Component)]
struct SettingsPanel;
#[derive(Component)]
struct SettingsButton;
#[derive(Component)]
struct RebindButton { action: Action }
#[derive(Component)]
struct RebindLabel { action: Action }

// ---------- actions & keybinds (keys are now DATA) ----------
#[derive(Clone, Copy, PartialEq, Eq)]
enum Action { Up, Down, Left, Right, Inventory }

const ACTIONS: [(Action, &str); 5] = [
    (Action::Up, "Move up"),
    (Action::Down, "Move down"),
    (Action::Left, "Move left"),
    (Action::Right, "Move right"),
    (Action::Inventory, "Open inventory"),
];

#[derive(Resource)]
struct Keybinds {
    up: KeyCode,
    down: KeyCode,
    left: KeyCode,
    right: KeyCode,
    inventory: KeyCode,
}
impl Default for Keybinds {
    fn default() -> Self {
        Keybinds {
            up: KeyCode::KeyW,
            down: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
            inventory: KeyCode::KeyE,
        }
    }
}
impl Keybinds {
    fn get(&self, a: Action) -> KeyCode {
        match a {
            Action::Up => self.up,
            Action::Down => self.down,
            Action::Left => self.left,
            Action::Right => self.right,
            Action::Inventory => self.inventory,
        }
    }
    fn set(&mut self, a: Action, k: KeyCode) {
        match a {
            Action::Up => self.up = k,
            Action::Down => self.down = k,
            Action::Left => self.left = k,
            Action::Right => self.right = k,
            Action::Inventory => self.inventory = k,
        }
    }
}

// turn KeyCode::KeyW into "W", Digit1 into "1", etc.
fn key_name(k: KeyCode) -> String {
    let s = format!("{:?}", k);
    s.strip_prefix("Key").map(|x| x.to_string())
        .or_else(|| s.strip_prefix("Digit").map(|x| x.to_string()))
        .unwrap_or(s)
}

#[derive(Resource, Default)]
struct Rebinding(Option<Action>);   // which action is waiting for a new key

// ---------- inventory data (items removed for now) ----------
#[allow(dead_code)] // we'll start putting items in slots soon
#[derive(Clone)]
struct ItemStack {
    name: String,
    count: u32,
}

#[derive(Resource)]
struct Inventory {
    slots: Vec<Option<ItemStack>>,
    selected: usize,
}
impl Default for Inventory {
    fn default() -> Self {
        Inventory { slots: vec![None; 36], selected: 0 }   // 0..9 hotbar, 9..36 inventory
    }
}

#[derive(Resource, Default)]
struct InventoryOpen(bool);
#[derive(Resource, Default)]
struct SettingsOpen(bool);

// ---------- world ----------
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
            for i in 0..9 { spawn_slot(bar, i); }
        });
    });
}

fn setup_inventory(mut commands: Commands) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
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
                    for c in 0..9 { spawn_slot(row, 9 + r * 9 + c); }
                });
            }
        });
    });
}

fn setup_settings(mut commands: Commands) {
    // top-left settings button (always visible)
    commands.spawn((
        Button,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            top: Val::Px(12.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.2, 0.24, 0.9)),
        SettingsButton,
    ))
    .with_children(|b| {
        b.spawn((Text::new("Settings"), TextFont { font_size: 13.0, ..default() }, TextColor(Color::WHITE)));
    });

    // the settings screen (hidden until clicked)
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        SettingsPanel,
    ))
    .with_children(|backdrop| {
        backdrop.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(16.0)),
                min_width: Val::Px(300.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.12, 0.15, 0.98)),
        ))
        .with_children(|panel| {
            panel.spawn((Text::new("Keybinds"), TextFont { font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
            for (action, label) in ACTIONS {
                panel.spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((Text::new(label), TextFont { font_size: 13.0, ..default() }, TextColor(Color::srgb(0.8, 0.8, 0.85))));
                    row.spawn((
                        Button,
                        Node {
                            width: Val::Px(74.0),
                            height: Val::Px(28.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.25, 0.25, 0.3, 1.0)),
                        RebindButton { action },
                    ))
                    .with_children(|btn| {
                        btn.spawn((Text::new(""), TextFont { font_size: 12.0, ..default() }, TextColor(Color::WHITE), RebindLabel { action }));
                    });
                });
            }
            panel.spawn((Text::new("click a key, then press the new key  (Esc cancels / closes)"),
                TextFont { font_size: 11.0, ..default() }, TextColor(Color::srgb(0.55, 0.55, 0.6))));
        });
    });
}

// ---------- logic ----------
fn select_hotbar_slot(
    keys: Res<ButtonInput<KeyCode>>,
    settings: Res<SettingsOpen>,
    mut inv: ResMut<Inventory>,
) {
    if settings.0 { return; }
    let digits = [
        KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3,
        KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6,
        KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9,
    ];
    for (i, key) in digits.iter().enumerate() {
        if keys.just_pressed(*key) { inv.selected = i; }
    }
}

fn toggle_inventory(
    keys: Res<ButtonInput<KeyCode>>,
    binds: Res<Keybinds>,
    settings: Res<SettingsOpen>,
    mut open: ResMut<InventoryOpen>,
) {
    if settings.0 { return; }
    if keys.just_pressed(binds.inventory) { open.0 = !open.0; }
}

fn apply_inventory_visibility(
    open: Res<InventoryOpen>,
    mut panel: Query<&mut Node, With<InventoryPanel>>,
) {
    if !open.is_changed() { return; }
    for mut node in &mut panel {
        node.display = if open.0 { Display::Flex } else { Display::None };
    }
}

fn settings_button(
    q: Query<&Interaction, (Changed<Interaction>, With<SettingsButton>)>,
    mut open: ResMut<SettingsOpen>,
) {
    for interaction in &q {
        if *interaction == Interaction::Pressed { open.0 = !open.0; }
    }
}

fn apply_settings_visibility(
    open: Res<SettingsOpen>,
    mut panel: Query<&mut Node, With<SettingsPanel>>,
) {
    if !open.is_changed() { return; }
    for mut node in &mut panel {
        node.display = if open.0 { Display::Flex } else { Display::None };
    }
}

fn rebind_button(
    q: Query<(&Interaction, &RebindButton), Changed<Interaction>>,
    mut rebinding: ResMut<Rebinding>,
) {
    for (interaction, btn) in &q {
        if *interaction == Interaction::Pressed { rebinding.0 = Some(btn.action); }
    }
}

fn capture_rebind(
    keys: Res<ButtonInput<KeyCode>>,
    mut rebinding: ResMut<Rebinding>,
    mut binds: ResMut<Keybinds>,
) {
    let Some(action) = rebinding.0 else { return; };
    // take the next pressed key that isn't Escape
    if let Some(&key) = keys.get_just_pressed().find(|k| **k != KeyCode::Escape) {
        binds.set(action, key);
        rebinding.0 = None;
    }
}

fn escape_handler(
    keys: Res<ButtonInput<KeyCode>>,
    mut rebinding: ResMut<Rebinding>,
    mut settings: ResMut<SettingsOpen>,
    mut inv: ResMut<InventoryOpen>,
) {
    if !keys.just_pressed(KeyCode::Escape) { return; }
    if rebinding.0.is_some() { rebinding.0 = None; }
    else if settings.0 { settings.0 = false; }
    else if inv.0 { inv.0 = false; }
}

fn update_rebind_labels(
    binds: Res<Keybinds>,
    rebinding: Res<Rebinding>,
    mut labels: Query<(&RebindLabel, &mut Text)>,
) {
    for (label, mut text) in &mut labels {
        text.0 = if rebinding.0 == Some(label.action) {
            "press…".to_string()
        } else {
            key_name(binds.get(label.action))
        };
    }
}

fn update_slots(
    inv: Res<Inventory>,
    mut slots: Query<(&Slot, &mut BackgroundColor)>,
    mut labels: Query<(&SlotLabel, &mut Text)>,
) {
    for (slot, mut bg) in &mut slots {
        bg.0 = if slot.index == inv.selected {
            Color::srgba(0.95, 0.78, 0.30, 0.95)
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

// ---------- movement & camera ----------
fn move_player(
    keys: Res<ButtonInput<KeyCode>>,
    binds: Res<Keybinds>,
    inv_open: Res<InventoryOpen>,
    settings: Res<SettingsOpen>,
    time: Res<Time>,
    mut players: Query<&mut Transform, With<Player>>,
) {
    if inv_open.0 || settings.0 { return; }   // frozen while a menu is open
    let speed = 300.0;
    let dt = time.delta_secs();
    for mut t in &mut players {
        if keys.pressed(binds.up) { t.translation.y += speed * dt; }
        if keys.pressed(binds.down) { t.translation.y -= speed * dt; }
        if keys.pressed(binds.left) { t.translation.x -= speed * dt; }
        if keys.pressed(binds.right) { t.translation.x += speed * dt; }
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