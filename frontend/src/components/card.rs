use leptos::prelude::*;
use shared::card::*;

#[component]
pub fn CardFace(card: Card) -> impl IntoView {
    let class = format!(
        "shape {} {}",
        color_class(card.color),
        fill_class(card.fill)
    );
    let ys = y_positions(card.count);

    view! {
        <svg viewBox="0 0 200 300" class="card-svg">
            <rect class="card-bg" width="200" height="300" rx="16"/>
            {ys
                .iter()
                .map(|&y| view! {
                    <g transform=format!("translate(100 {y})") class=class.clone()>
                        {shape_view(card.shape)}
                    </g>
                })
                .collect_view()}
        </svg>
    }
}

fn shape_view(shape: Shape) -> AnyView {
    match shape {
        Shape::Diamond => view! {
            <path d="M -75 0 L 0 -30 L 75 0 L 0 30 Z"/>
        }
        .into_any(),
        Shape::Oval => view! {
            <rect x="-75" y="-30" width="150" height="60" rx="30"/>
        }
        .into_any(),
        Shape::Squiggle => view! {
            <path d="M -72 14 C -82 -14 -60 -34 -30 -26 C -6 -20 12 -8 36 -20 C 58 -32 80 -28 76 -6 C 72 18 48 30 22 24 C -2 18 -18 8 -42 22 C -60 32 -68 26 -72 14 Z"/>
        }
        .into_any(),
    }
}

fn color_class(color: Color) -> &'static str {
    match color {
        Color::Red => "red",
        Color::Green => "green",
        Color::Purple => "purple",
    }
}

fn fill_class(fill: Fill) -> &'static str {
    match fill {
        Fill::Empty => "empty",
        Fill::Striped => "striped",
        Fill::Solid => "solid",
    }
}

fn y_positions(count: Count) -> &'static [i32] {
    match count {
        Count::One => &[150],
        Count::Two => &[115, 185],
        Count::Three => &[80, 150, 220],
    }
}
