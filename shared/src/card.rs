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

pub fn is_set_ids(ids: [u8; 3]) -> bool {
    let [a, b, c] = ids.map(Card::from_id);
    is_set(&a, &b, &c)
}

impl Card {
    pub fn from_id(id: u8) -> Card {
        assert!(id < 81);
        Card {
            shape: Shape::ALL[(id / 27) as usize],
            count: Count::ALL[(id / 9 % 3) as usize],
            color: Color::ALL[(id / 3 % 3) as usize],
            fill:  Fill::ALL[(id % 3) as usize],
        }
    }

    pub fn id(&self) -> u8 {
        self.shape as u8 * 27 + self.count as u8 * 9 + self.color as u8 * 3 + self.fill as u8
    }
}


impl Shape {
    pub const ALL: [Shape; 3] = [Shape::Diamond, Shape::Oval, Shape::Squiggle];
}

impl Count {
    pub const ALL: [Count; 3] = [Count::One, Count::Two, Count::Three];
}

impl Color {
    pub const ALL: [Color; 3] = [Color::Red, Color::Purple, Color::Green];
}

impl Fill {
    pub const ALL: [Fill; 3] = [Fill::Solid, Fill::Striped, Fill::Empty];
}