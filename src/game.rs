use aabb::Aabb;
use cgmath::{vec2, InnerSpace, Vector2};
use fnv::FnvHashMap;

#[derive(Debug)]
pub struct InputModel {
    vec: Vector2<f32>,
}

impl Default for InputModel {
    fn default() -> Self {
        Self {
            vec: vec2(0., 0.),
        }
    }
}

impl InputModel {
    pub fn set_x(&mut self, value: f32) {
        self.vec.x = value
    }
    pub fn set_y(&mut self, value: f32) {
        self.vec.y = value
    }
    pub fn vector(&self) -> Vector2<f32> {
        if self.vec.magnitude2() > 1. {
            self.vec.normalize()
        } else {
            self.vec
        }
    }
}

type EntityId = u16;

#[derive(Default)]
struct EntityIdAllocator {
    next: EntityId,
}

impl EntityIdAllocator {
    fn allocate(&mut self) -> EntityId {
        let id = self.next;
        self.next += 1;
        id
    }
}

pub struct GameState {
    aabb: FnvHashMap<EntityId, Aabb>,
    colour: FnvHashMap<EntityId, [f32; 3]>,
    player_id: EntityId,
    entity_id_allocator: EntityIdAllocator,
}

impl GameState {
    pub fn new() -> Self {
        let mut entity_id_allocator = EntityIdAllocator::default();
        let player_id = entity_id_allocator.allocate();
        let mut game_state = Self {
            player_id,
            entity_id_allocator,
            aabb: Default::default(),
            colour: Default::default(),
        };
        game_state.aabb.insert(
            player_id,
            Aabb {
                top_left_coord: vec2(40., 60.),
                size: vec2(10., 16.),
            },
        );
        game_state.colour.insert(player_id, [1., 0., 0.]);
        game_state
    }

    pub fn to_render(&self) -> impl Iterator<Item = ToRender> {
        self.aabb.iter().filter_map(move |(id, aabb)| {
            self.colour
                .get(id)
                .map(|&colour| ToRender { aabb, colour })
        })
    }

    pub fn update(&mut self, input_model: &InputModel) {
        if let Some(aabb) = self.aabb.get_mut(&self.player_id) {
            aabb.top_left_coord += input_model.vector();
        }
    }
}

pub struct ToRender<'a> {
    pub aabb: &'a Aabb,
    pub colour: [f32; 3],
}
