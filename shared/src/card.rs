use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Shape {
    Diamond,
    Oval,
    Squiggle,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Count {
    One,
    Two,
    Three,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Color {
    Red,
    Purple,
    Green,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
#[repr(u8)]
pub enum Fill {
    Solid,
    Striped,
    Empty,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Card {
    pub shape: Shape,
    pub count: Count,
    pub color: Color,
    pub fill: Fill,
}

fn ok(a: u8, b: u8, c: u8) -> bool {
    (a + b + c).is_multiple_of(3)
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

/// The value that completes a set attribute: the three values must sum to a multiple of 3.
/// `6 - a - b` stays non-negative for values 0..=2.
fn missing_value(a: u8, b: u8) -> u8 {
    (6 - a - b) % 3
}

/// The only card that forms a set with `a` and `b`.
pub fn missing_card(a: &Card, b: &Card) -> Card {
    Card {
        shape: Shape::ALL[missing_value(a.shape as u8, b.shape as u8) as usize],
        count: Count::ALL[missing_value(a.count as u8, b.count as u8) as usize],
        color: Color::ALL[missing_value(a.color as u8, b.color as u8) as usize],
        fill: Fill::ALL[missing_value(a.fill as u8, b.fill as u8) as usize],
    }
}

/// Same as [`missing_card`], computed straight from card IDs.
///
/// An ID is the four attributes written as base-3 digits (`shape*27 + count*9 + color*3 + fill`),
/// so the missing card's ID is built digit by digit without decoding to a `Card`.
pub fn missing_id(a: u8, b: u8) -> u8 {
    debug_assert!(a < 81 && b < 81);
    [27, 9, 3, 1]
        .iter()
        .map(|&place| missing_value(a / place % 3, b / place % 3) * place)
        .sum()
}

impl Card {
    pub fn from_id(id: u8) -> Card {
        assert!(id < 81);
        Card {
            shape: Shape::ALL[(id / 27) as usize],
            count: Count::ALL[(id / 9 % 3) as usize],
            color: Color::ALL[(id / 3 % 3) as usize],
            fill: Fill::ALL[(id % 3) as usize],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_id_examples() {
        // 0, 1, 2 differ only in fill.
        assert_eq!(missing_id(0, 1), 2);
        assert_eq!(missing_id(0, 2), 1);
        assert_eq!(missing_id(1, 2), 0);
    }

    #[test]
    fn missing_id_completes_every_pair() {
        for a in 0..81 {
            for b in 0..81 {
                if a == b {
                    continue;
                }
                let c = missing_id(a, b);
                assert!(c != a && c != b, "{a} {b} -> {c}");
                assert!(is_set_ids([a, b, c]), "{a} {b} -> {c}");
                assert_eq!(c, missing_card(&Card::from_id(a), &Card::from_id(b)).id());
            }
        }
    }
}
