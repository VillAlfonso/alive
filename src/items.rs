#![allow(dead_code)] // these definitions get wired up when eating/combat land

use bevy::prelude::*;
use std::collections::HashMap;

// effects a consumable can apply when used
#[derive(Clone, Copy)]
pub enum Effect {
    Regen  { per_second: f32, seconds: f32 },
    Speed  { multiplier: f32, seconds: f32 },
    Poison { per_second: f32, seconds: f32 },
}

// the "tag" of an item — what category it is and the data that category needs
#[derive(Clone)]
pub enum ItemKind {
    Consumable { restore_health: f32, restore_hunger: f32, effects: Vec<Effect> },
    Weapon     { damage: f32 },
    Material,
}

// one entry in the item dictionary
#[derive(Clone)]
pub struct ItemDef {
    pub id: &'static str,
    pub name: &'static str,
    pub max_stack: u32,
    pub kind: ItemKind,
}

// the dictionary itself, looked up by id — lives as a Resource
#[derive(Resource)]
pub struct ItemDb {
    pub items: HashMap<&'static str, ItemDef>,
}

impl ItemDb {
    pub fn get(&self, id: &str) -> Option<&ItemDef> {
        self.items.get(id)
    }
}

// the master list. adding an item = one entry here.
pub fn item_database() -> ItemDb {
    let defs = vec![
        ItemDef {
            id: "apple", name: "Apple", max_stack: 64,
            kind: ItemKind::Consumable { restore_health: 0.0, restore_hunger: 4.0, effects: vec![] },
        },
        ItemDef {
            id: "cooked_meat", name: "Cooked Meat", max_stack: 64,
            kind: ItemKind::Consumable { restore_health: 0.0, restore_hunger: 8.0, effects: vec![] },
        },
        ItemDef {
            id: "healing_herb", name: "Healing Herb", max_stack: 16,
            kind: ItemKind::Consumable {
                restore_health: 6.0, restore_hunger: 0.0,
                effects: vec![Effect::Regen { per_second: 1.0, seconds: 4.0 }],
            },
        },
        ItemDef {
            id: "iron_sword", name: "Iron Sword", max_stack: 1,
            kind: ItemKind::Weapon { damage: 6.0 },
        },
        ItemDef { id: "stone", name: "Stone", max_stack: 64, kind: ItemKind::Material },
        ItemDef { id: "wood",  name: "Wood",  max_stack: 64, kind: ItemKind::Material },
    ];

    let mut items = HashMap::new();
    for d in defs {
        items.insert(d.id, d);
    }
    ItemDb { items }
}