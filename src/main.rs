mod items;
mod npc;   // <-- NPC test (delete this line + npc.rs to remove)
mod dialogue;

use bevy::prelude::*;
use std::collections::HashMap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(npc::NpcPlugin)   // <-- NPC test (delete this line to remove)
        .add_plugins(dialogue::DialoguePlugin)
        .insert_resource(items::item_database())
        .init_resource::<Inventory>()
        .init_resource::<InventoryOpen>()
        .init_resource::<SettingsOpen>()
        .init_resource::<Keybinds>()
        .init_resource::<Rebinding>()
        .init_resource::<Health>()
        .init_resource::<Hunger>()
        .init_resource::<Held>()
        .add_systems(Startup, (setup, setup_hotbar, setup_inventory, setup_settings, setup_held))
        .add_systems(Update, (player_movement, follow_player_with_camera).chain())
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
            click_slot,
            update_held_display,
            update_slots,
            update_stats_bars,
        ))
        .run();
}

// ---------- movement feel ----------
const WALK_SPEED: f32 = 300.0;
const SPRINT_SPEED: f32 = 480.0;
const SNEAK_SPEED: f32 = 140.0;
const DODGE_SPEED: f32 = 900.0;
const DODGE_DURATION: f32 = 0.22;
const DODGE_COOLDOWN: f32 = 0.5;

// ---------- components ----------
#[derive(Component)]
struct Player;
#[derive(Component, Default)]
struct DodgeState { timer: f32, cooldown: f32, dir: Vec2 }
#[derive(Component)]
struct Slot { index: usize }
#[derive(Component)]
struct SlotLabel { index: usize }
#[derive(Component)]
struct Pip { health: bool, index: usize }
#[derive(Component)]
struct HeldDisplay;
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

// ---------- actions & keybinds ----------
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Action {
    Up, Down, Left, Right,
    Sprint, Sneak, DodgeRoll, Inventory, Talk, Chat,
    Hotbar1, Hotbar2, Hotbar3, Hotbar4, Hotbar5,
    Hotbar6, Hotbar7, Hotbar8, Hotbar9,
}

const ACTIONS: [(Action, &str, KeyCode); 20] = [
    (Action::Up,        "Move up",     KeyCode::KeyW),
    (Action::Down,      "Move down",   KeyCode::KeyS),
    (Action::Left,      "Move left",   KeyCode::KeyA),
    (Action::Right,     "Move right",  KeyCode::KeyD),
    (Action::Sprint,    "Sprint",      KeyCode::ControlLeft),
    (Action::Sneak,     "Sneak",       KeyCode::ShiftLeft),
    (Action::DodgeRoll, "Dodge roll",  KeyCode::Space),
    (Action::Inventory, "Inventory",   KeyCode::KeyE),
    (Action::Talk,      "Talk / shout", KeyCode::KeyF),
    (Action::Hotbar1,   "Hotbar 1",    KeyCode::Digit1),
    (Action::Hotbar2,   "Hotbar 2",    KeyCode::Digit2),
    (Action::Hotbar3,   "Hotbar 3",    KeyCode::Digit3),
    (Action::Hotbar4,   "Hotbar 4",    KeyCode::Digit4),
    (Action::Hotbar5,   "Hotbar 5",    KeyCode::Digit5),
    (Action::Hotbar6,   "Hotbar 6",    KeyCode::Digit6),
    (Action::Hotbar7,   "Hotbar 7",    KeyCode::Digit7),
    (Action::Hotbar8,   "Hotbar 8",    KeyCode::Digit8),
    (Action::Hotbar9,   "Hotbar 9",    KeyCode::Digit9),
        (Action::Talk,      "Talk / shout", KeyCode::KeyF),
    (Action::Chat,      "Reply / type", KeyCode::KeyT),
];

#[derive(Resource)]
struct Keybinds { map: HashMap<Action, KeyCode> }
impl Default for Keybinds {
    fn default() -> Self {
        let mut map = HashMap::new();
        for (action, _label, key) in ACTIONS { map.insert(action, key); }
        Keybinds { map }
    }
}
impl Keybinds {
    fn get(&self, a: Action) -> KeyCode { self.map.get(&a).copied().unwrap_or(KeyCode::Escape) }
    fn set(&mut self, a: Action, k: KeyCode) { self.map.insert(a, k); }
}

fn key_name(k: KeyCode) -> String {
    use KeyCode::*;
    match k {
        Space => "Space".into(),
        ShiftLeft => "L Shift".into(), ShiftRight => "R Shift".into(),
        ControlLeft => "L Ctrl".into(), ControlRight => "R Ctrl".into(),
        AltLeft => "L Alt".into(),
        ArrowUp => "Up".into(), ArrowDown => "Down".into(),
        ArrowLeft => "Left".into(), ArrowRight => "Right".into(),
        other => {
            let s = format!("{:?}", other);
            s.strip_prefix("Key").map(|x| x.to_string())
                .or_else(|| s.strip_prefix("Digit").map(|x| x.to_string()))
                .unwrap_or(s)
        }
    }
}

#[derive(Resource, Default)]
struct Rebinding(Option<Action>);

// ---------- inventory & stats data ----------
#[derive(Clone)]
struct ItemStack { id: &'static str, count: u32 }

#[derive(Resource)]
struct Inventory { slots: Vec<Option<ItemStack>>, selected: usize }
impl Default for Inventory {
    fn default() -> Self {
        let mut slots: Vec<Option<ItemStack>> = vec![None; 37];
        slots[0]  = Some(ItemStack { id: "iron_sword",   count: 1 });
        slots[9]  = Some(ItemStack { id: "apple",        count: 5 });
        slots[10] = Some(ItemStack { id: "stone",        count: 30 });
        slots[11] = Some(ItemStack { id: "cooked_meat",  count: 3 });
        slots[12] = Some(ItemStack { id: "healing_herb", count: 2 });
        Inventory { slots, selected: 0 }
    }
}

#[derive(Resource, Default)]
struct Held(Option<ItemStack>);

#[derive(Resource, Default)]
struct InventoryOpen(bool);
#[derive(Resource, Default)]
struct SettingsOpen(bool);

#[allow(dead_code)]
#[derive(Resource)]
struct Health { current: f32, max: f32 }
impl Default for Health { fn default() -> Self { Health { current: 20.0, max: 20.0 } } }

#[allow(dead_code)]
#[derive(Resource)]
struct Hunger { current: f32, max: f32 }
impl Default for Hunger { fn default() -> Self { Hunger { current: 20.0, max: 20.0 } } }

// ---------- world ----------
fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite::from_color(Color::srgb(1.0, 0.48, 0.27), Vec2::new(24.0, 24.0)),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Player,
        DodgeState::default(),
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
        Button,
        Node { width: Val::Px(50.0), height: Val::Px(50.0),
            justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
        BackgroundColor(Color::srgba(0.25, 0.25, 0.28, 0.85)),
        Slot { index },
    ))
    .with_children(|slot| {
        slot.spawn((
            Text::new(""), TextFont { font_size: 11.0, ..default() }, TextColor(Color::WHITE),
            TextLayout { justify: Justify::Center, ..default() }, SlotLabel { index },
        ));
    });
}

fn spawn_pip(parent: &mut ChildSpawnerCommands, health: bool, index: usize) {
    parent.spawn((
        Node { width: Val::Px(14.0), height: Val::Px(14.0), ..default() },
        BackgroundColor(Color::srgba(0.2, 0.2, 0.22, 0.9)),
        Pip { health, index },
    ));
}

fn setup_hotbar(mut commands: Commands) {
    commands.spawn(Node {
        position_type: PositionType::Absolute,
        left: Val::Px(0.0), right: Val::Px(0.0), bottom: Val::Px(16.0),
        justify_content: JustifyContent::Center, ..default()
    })
    .with_children(|root| {
        root.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(6.0), ..default() })
            .with_children(|col| {
                col.spawn(Node { width: Val::Px(490.0), flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, ..default() })
                    .with_children(|stats| {
                        stats.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(3.0), ..default() })
                            .with_children(|hp| { for i in 0..10 { spawn_pip(hp, true, i); } });
                        stats.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(3.0), ..default() })
                            .with_children(|hg| { for i in 0..10 { spawn_pip(hg, false, i); } });
                    });
                col.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(12.0), ..default() })
                    .with_children(|row| {
                        row.spawn((Node { padding: UiRect::all(Val::Px(4.0)), ..default() }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45))))
                            .with_children(|oh| { spawn_slot(oh, 36); });
                        row.spawn((Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(4.0), padding: UiRect::all(Val::Px(4.0)), ..default() }, BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45))))
                            .with_children(|bar| { for i in 0..9 { spawn_slot(bar, i); } });
                    });
            });
    });
}

fn setup_inventory(mut commands: Commands) {
    commands.spawn((
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0),
            justify_content: JustifyContent::Center, align_items: AlignItems::Center,
            display: Display::None, ..default() },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        InventoryPanel,
    ))
    .with_children(|backdrop| {
        backdrop.spawn((
            Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center,
                row_gap: Val::Px(8.0), padding: UiRect::all(Val::Px(14.0)), ..default() },
            BackgroundColor(Color::srgba(0.12, 0.12, 0.15, 0.98)),
        ))
        .with_children(|panel| {
            panel.spawn((Text::new("Inventory"), TextFont { font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
            for r in 0..3 {
                panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(4.0), ..default() })
                    .with_children(|row| { for c in 0..9 { spawn_slot(row, 9 + r * 9 + c); } });
            }
            panel.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: Val::Px(12.0), padding: UiRect { top: Val::Px(6.0), ..default() }, ..default() })
                .with_children(|row| {
                    spawn_slot(row, 36);
                    row.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(4.0), ..default() })
                        .with_children(|bar| { for i in 0..9 { spawn_slot(bar, i); } });
                });
        });
    });
}

fn setup_held(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::WHITE),
        Node { position_type: PositionType::Absolute, display: Display::None, padding: UiRect::all(Val::Px(3.0)), ..default() },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.12, 0.95)),
        HeldDisplay,
    ));
}

fn build_keybind_column(parent: &mut ChildSpawnerCommands, rows: &[(Action, &str, KeyCode)]) {
    parent.spawn(Node { flex_direction: FlexDirection::Column, row_gap: Val::Px(6.0), ..default() })
        .with_children(|col| {
            for (action, label, _key) in rows {
                col.spawn(Node { width: Val::Px(220.0), flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, column_gap: Val::Px(16.0), ..default() })
                    .with_children(|row| {
                        row.spawn((Text::new(*label), TextFont { font_size: 13.0, ..default() }, TextColor(Color::srgb(0.8, 0.8, 0.85))));
                        row.spawn((
                            Button,
                            Node { width: Val::Px(70.0), height: Val::Px(26.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                            BackgroundColor(Color::srgba(0.25, 0.25, 0.3, 1.0)),
                            RebindButton { action: *action },
                        ))
                        .with_children(|btn| {
                            btn.spawn((Text::new(""), TextFont { font_size: 12.0, ..default() }, TextColor(Color::WHITE), RebindLabel { action: *action }));
                        });
                    });
            }
        });
}

fn setup_settings(mut commands: Commands) {
    commands.spawn((
        Button,
        Node { position_type: PositionType::Absolute, left: Val::Px(12.0), top: Val::Px(12.0), padding: UiRect::all(Val::Px(8.0)), ..default() },
        BackgroundColor(Color::srgba(0.2, 0.2, 0.24, 0.9)),
        SettingsButton,
    ))
    .with_children(|b| { b.spawn((Text::new("Settings"), TextFont { font_size: 13.0, ..default() }, TextColor(Color::WHITE))); });

    commands.spawn((
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, display: Display::None, ..default() },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        SettingsPanel,
    ))
    .with_children(|backdrop| {
        backdrop.spawn((
            Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: Val::Px(10.0), padding: UiRect::all(Val::Px(16.0)), ..default() },
            BackgroundColor(Color::srgba(0.12, 0.12, 0.15, 0.98)),
        ))
        .with_children(|panel| {
            panel.spawn((Text::new("Keybinds"), TextFont { font_size: 18.0, ..default() }, TextColor(Color::WHITE)));
            panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(28.0), align_items: AlignItems::FlexStart, ..default() })
                .with_children(|cols| {
                     build_keybind_column(cols, &ACTIONS[0..10]);
                    build_keybind_column(cols, &ACTIONS[10..19]);
                });
            panel.spawn((Text::new("click a key, then press the new key  (Esc cancels / closes)"), TextFont { font_size: 11.0, ..default() }, TextColor(Color::srgb(0.55, 0.55, 0.6))));
        });
    });
}

// ---------- click-to-move inventory ----------
fn resolve_click(slot: Option<ItemStack>, held: Option<ItemStack>, db: &items::ItemDb)
    -> (Option<ItemStack>, Option<ItemStack>)
{
    match (slot, held) {
        (s, None) => (None, s),
        (None, h) => (h, None),
        (Some(mut s), Some(h)) => {
            if s.id == h.id {
                let max = db.get(s.id).map(|d| d.max_stack).unwrap_or(64);
                let moved = max.saturating_sub(s.count).min(h.count);
                s.count += moved;
                let leftover = h.count - moved;
                if leftover > 0 { (Some(s), Some(ItemStack { id: h.id, count: leftover })) }
                else { (Some(s), None) }
            } else {
                (Some(h), Some(s))
            }
        }
    }
}

fn click_slot(
    inv_open: Res<InventoryOpen>,
    db: Res<items::ItemDb>,
    mut inv: ResMut<Inventory>,
    mut held: ResMut<Held>,
    q: Query<(&Interaction, &Slot), Changed<Interaction>>,
) {
    if !inv_open.0 { return; }
    for (interaction, slot) in &q {
        if *interaction != Interaction::Pressed { continue; }
        let i = slot.index;
        let slot_item = inv.slots[i].take();
        let held_item = held.0.take();
        let (new_slot, new_held) = resolve_click(slot_item, held_item, &db);
        inv.slots[i] = new_slot;
        held.0 = new_held;
    }
}

fn update_held_display(
    held: Res<Held>,
    inv_open: Res<InventoryOpen>,
    db: Res<items::ItemDb>,
    windows: Query<&Window>,
    mut q: Query<(&mut Node, &mut Text), With<HeldDisplay>>,
) {
    let Ok((mut node, mut text)) = q.single_mut() else { return; };
    if !inv_open.0 || held.0.is_none() {
        node.display = Display::None;
        return;
    }
    node.display = Display::Flex;
    if let Some(stack) = &held.0 {
        let name = db.get(stack.id).map(|d| d.name).unwrap_or(stack.id);
        text.0 = if stack.count > 1 { format!("{} x{}", name, stack.count) } else { name.to_string() };
    }
    if let Ok(window) = windows.single() {
        if let Some(cursor) = window.cursor_position() {
            node.left = Val::Px(cursor.x + 12.0);
            node.top = Val::Px(cursor.y + 12.0);
        }
    }
}

// ---------- other logic ----------
fn select_hotbar_slot( input_lock: Res<dialogue::InputLock>,keys: Res<ButtonInput<KeyCode>>, binds: Res<Keybinds>, settings: Res<SettingsOpen>, mut inv: ResMut<Inventory>) {
     if settings.0 || input_lock.0 { return; }
    let slots = [
        Action::Hotbar1, Action::Hotbar2, Action::Hotbar3, Action::Hotbar4, Action::Hotbar5,
        Action::Hotbar6, Action::Hotbar7, Action::Hotbar8, Action::Hotbar9,
    ];
    for (i, a) in slots.iter().enumerate() {
        if keys.just_pressed(binds.get(*a)) { inv.selected = i; }
    }
}

fn toggle_inventory( input_lock: Res<dialogue::InputLock>,keys: Res<ButtonInput<KeyCode>>, binds: Res<Keybinds>, settings: Res<SettingsOpen>, mut open: ResMut<InventoryOpen>) {
    if settings.0 || input_lock.0 { return; }
    if keys.just_pressed(binds.get(Action::Inventory)) { open.0 = !open.0; }
}

fn apply_inventory_visibility(open: Res<InventoryOpen>, mut panel: Query<&mut Node, With<InventoryPanel>>) {
    if !open.is_changed() { return; }
    for mut node in &mut panel { node.display = if open.0 { Display::Flex } else { Display::None }; }
}

fn settings_button(q: Query<&Interaction, (Changed<Interaction>, With<SettingsButton>)>, mut open: ResMut<SettingsOpen>) {
    for interaction in &q { if *interaction == Interaction::Pressed { open.0 = !open.0; } }
}

fn apply_settings_visibility(open: Res<SettingsOpen>, mut panel: Query<&mut Node, With<SettingsPanel>>) {
    if !open.is_changed() { return; }
    for mut node in &mut panel { node.display = if open.0 { Display::Flex } else { Display::None }; }
}

fn rebind_button(q: Query<(&Interaction, &RebindButton), Changed<Interaction>>, mut rebinding: ResMut<Rebinding>) {
    for (interaction, btn) in &q { if *interaction == Interaction::Pressed { rebinding.0 = Some(btn.action); } }
}

fn capture_rebind(keys: Res<ButtonInput<KeyCode>>, mut rebinding: ResMut<Rebinding>, mut binds: ResMut<Keybinds>) {
    let Some(action) = rebinding.0 else { return; };
    if let Some(&key) = keys.get_just_pressed().find(|k| **k != KeyCode::Escape) {
        binds.set(action, key);
        rebinding.0 = None;
    }
}

fn escape_handler(keys: Res<ButtonInput<KeyCode>>, mut rebinding: ResMut<Rebinding>, mut settings: ResMut<SettingsOpen>, mut inv: ResMut<InventoryOpen>) {
    if !keys.just_pressed(KeyCode::Escape) { return; }
    if rebinding.0.is_some() { rebinding.0 = None; }
    else if settings.0 { settings.0 = false; }
    else if inv.0 { inv.0 = false; }
}

fn update_rebind_labels(binds: Res<Keybinds>, rebinding: Res<Rebinding>, mut labels: Query<(&RebindLabel, &mut Text)>) {
    for (label, mut text) in &mut labels {
        text.0 = if rebinding.0 == Some(label.action) { "press…".to_string() } else { key_name(binds.get(label.action)) };
    }
}

fn update_slots(
    db: Res<items::ItemDb>,
    inv: Res<Inventory>,
    mut slots: Query<(&Slot, &mut BackgroundColor)>,
    mut labels: Query<(&SlotLabel, &mut Text)>,
) {
    for (slot, mut bg) in &mut slots {
        bg.0 = if slot.index == inv.selected { Color::srgba(0.95, 0.78, 0.30, 0.95) }
               else { Color::srgba(0.25, 0.25, 0.28, 0.85) };
    }
    for (label, mut text) in &mut labels {
        text.0 = match &inv.slots[label.index] {
            Some(stack) => {
                let name = db.get(stack.id).map(|d| d.name).unwrap_or(stack.id);
                if stack.count > 1 { format!("{}\n{}", name, stack.count) } else { name.to_string() }
            }
            None => String::new(),
        };
    }
}

fn update_stats_bars(health: Res<Health>, hunger: Res<Hunger>, mut pips: Query<(&Pip, &mut BackgroundColor)>) {
    for (pip, mut bg) in &mut pips {
        let value = if pip.health { health.current } else { hunger.current };
        let full = value >= (pip.index as f32 + 1.0) * 2.0;
        bg.0 = if full {
            if pip.health { Color::srgb(0.86, 0.23, 0.23) } else { Color::srgb(0.77, 0.48, 0.18) }
        } else {
            Color::srgba(0.2, 0.2, 0.22, 0.9)
        };
    }
}

// ---------- movement ----------
fn player_movement(
    input_lock: Res<dialogue::InputLock>,
    keys: Res<ButtonInput<KeyCode>>, binds: Res<Keybinds>,
    inv_open: Res<InventoryOpen>, settings: Res<SettingsOpen>,
    time: Res<Time>, mut players: Query<(&mut Transform, &mut DodgeState), With<Player>>,
) {
    let dt = time.delta_secs();
    for (mut t, mut dodge) in &mut players {
        if dodge.cooldown > 0.0 { dodge.cooldown -= dt; }
       if inv_open.0 || settings.0 || input_lock.0 { dodge.timer = 0.0; continue; }

        let mut dir = Vec2::ZERO;
        if keys.pressed(binds.get(Action::Up))    { dir.y += 1.0; }
        if keys.pressed(binds.get(Action::Down))  { dir.y -= 1.0; }
        if keys.pressed(binds.get(Action::Left))  { dir.x -= 1.0; }
        if keys.pressed(binds.get(Action::Right)) { dir.x += 1.0; }

        if dodge.timer <= 0.0 && dodge.cooldown <= 0.0 && keys.just_pressed(binds.get(Action::DodgeRoll)) {
            dodge.dir = if dir != Vec2::ZERO { dir.normalize() } else { Vec2::new(0.0, -1.0) };
            dodge.timer = DODGE_DURATION;
            dodge.cooldown = DODGE_COOLDOWN;
        }

        if dodge.timer > 0.0 {
            t.translation.x += dodge.dir.x * DODGE_SPEED * dt;
            t.translation.y += dodge.dir.y * DODGE_SPEED * dt;
            dodge.timer -= dt;
        } else if dir != Vec2::ZERO {
            let speed = if keys.pressed(binds.get(Action::Sprint)) { SPRINT_SPEED }
                        else if keys.pressed(binds.get(Action::Sneak)) { SNEAK_SPEED }
                        else { WALK_SPEED };
            let v = dir.normalize() * speed * dt;
            t.translation.x += v.x;
            t.translation.y += v.y;
        }
    }
}

fn follow_player_with_camera(
    time: Res<Time>, players: Query<&Transform, With<Player>>,
    mut cameras: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok(player) = players.single() else { return; };
    let Ok(mut camera) = cameras.single_mut() else { return; };
    let lerp_speed = 5.0;
    let dt = time.delta_secs();
    camera.translation.x = camera.translation.x.lerp(player.translation.x, lerp_speed * dt);
    camera.translation.y = camera.translation.y.lerp(player.translation.y, lerp_speed * dt);
}