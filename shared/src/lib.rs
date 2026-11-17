use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Shape { Diamond, Oval, Squiggle }

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Count { One, Two, Three }

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Color { Red, Purple, Green }

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Fill { Solid, Striped, Empty }

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Card {
    pub shape: Shape,
    pub count: Count,
    pub color: Color,
    pub fill: Fill,
}

fn ok(a: u8, b: u8, c: u8) -> bool {
    (a + b + c) % 3 == 0
}

pub fn is_set(a: &Card, b: &Card, c: &Card) -> bool {
    ok(a.shape as u8, b.shape as u8, c.shape as u8)
        && ok(a.count as u8, b.count as u8, c.count as u8)
        && ok(a.color as u8, b.color as u8, c.color as u8)
        && ok(a.fill as u8, b.fill as u8, c.fill as u8)
}