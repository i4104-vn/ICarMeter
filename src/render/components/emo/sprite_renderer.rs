//! Renderer for each expression's horizontal SVG animation strip.

use super::EmoExpression;
use std::sync::OnceLock;
use tiny_skia::{Color, FillRule, Paint, Path, PathBuilder, Pixmap, Transform};

pub(super) const EXPRESSION_COUNT: usize = 35;
pub(super) const FRAME_COUNT: usize = 24;
pub(super) const FRAME_RATE: f32 = 6.0;
pub(super) const EXPRESSION_DURATION: f32 = FRAME_COUNT as f32 / FRAME_RATE;
const MAX_DISPLAY_WIDTH: f32 = 330.0;
const MAX_DISPLAY_HEIGHT: f32 = 125.0;

struct ExpressionSprite {
    frames: Vec<Vec<ColoredPath>>,
    bounds: Option<(f32, f32, f32, f32)>,
    frame_width: f32,
}

#[derive(Clone)]
struct ColoredPath {
    path: Path,
    color: Color,
    opacity: f32,
}

static EXPRESSIONS: OnceLock<Vec<ExpressionSprite>> = OnceLock::new();
const EXPRESSION_SVGS: [&str; EXPRESSION_COUNT] = [
    include_str!("../../../../assets/emotions/0.svg"),
    include_str!("../../../../assets/emotions/bee.svg"),
    include_str!("../../../../assets/emotions/chong_mat.svg"),
    include_str!("../../../../assets/emotions/cry.svg"),
    include_str!("../../../../assets/emotions/cry2.svg"),
    include_str!("../../../../assets/emotions/cuoi.svg"),
    include_str!("../../../../assets/emotions/demon_slayer.svg"),
    include_str!("../../../../assets/emotions/devil.svg"),
    include_str!("../../../../assets/emotions/distracted.svg"),
    include_str!("../../../../assets/emotions/egg.svg"),
    include_str!("../../../../assets/emotions/gian_du.svg"),
    include_str!("../../../../assets/emotions/happy_2.svg"),
    include_str!("../../../../assets/emotions/hat_xi.svg"),
    include_str!("../../../../assets/emotions/hat_xi_2.svg"),
    include_str!("../../../../assets/emotions/hehe.svg"),
    include_str!("../../../../assets/emotions/khoc.svg"),
    include_str!("../../../../assets/emotions/leu_leu.svg"),
    include_str!("../../../../assets/emotions/ngac_nhien.svg"),
    include_str!("../../../../assets/emotions/ngap.svg"),
    include_str!("../../../../assets/emotions/nheo_mat.svg"),
    include_str!("../../../../assets/emotions/nhin_ben_phai.svg"),
    include_str!("../../../../assets/emotions/nhin_ben_trai.svg"),
    include_str!("../../../../assets/emotions/nhin_xuong.svg"),
    include_str!("../../../../assets/emotions/rain.svg"),
    include_str!("../../../../assets/emotions/rainbow.svg"),
    include_str!("../../../../assets/emotions/scared.svg"),
    include_str!("../../../../assets/emotions/sleepy.svg"),
    include_str!("../../../../assets/emotions/smile.svg"),
    include_str!("../../../../assets/emotions/smirk.svg"),
    include_str!("../../../../assets/emotions/squint.svg"),
    include_str!("../../../../assets/emotions/sung_nuoc.svg"),
    include_str!("../../../../assets/emotions/sung_nuoc_2.svg"),
    include_str!("../../../../assets/emotions/thu_nho.svg"),
    include_str!("../../../../assets/emotions/turbo.svg"),
    include_str!("../../../../assets/emotions/UwU.svg"),
];

pub struct EmoSpriteRenderer;

impl EmoSpriteRenderer {
    pub fn draw(pixmap: &mut Pixmap, cx: f32, cy: f32, time: f32, alpha: f32, mood: EmoExpression) {
        if alpha <= 0.001 {
            return;
        }

        let expressions =
            EXPRESSIONS.get_or_init(|| EXPRESSION_SVGS.iter().map(|svg| parse_svg(svg)).collect());
        let expression_index = expression_cell(mood).min(expressions.len() - 1);
        let sprite = &expressions[expression_index];
        // Begin with a representative pose so forced emotions read correctly on the first frame.
        let elapsed_frame = (time.max(0.0) * FRAME_RATE).floor() as usize;
        let frame_index = if mood == EmoExpression::Sleeping
            || mood == EmoExpression::Sleepy
            || mood == EmoExpression::Atlas(26)
        {
            elapsed_frame % FRAME_COUNT
        } else {
            elapsed_frame.min(FRAME_COUNT - 1)
        };
        let frame = &sprite.frames[frame_index];
        let Some((left, top, right, bottom)) = sprite.bounds else {
            return;
        };
        let width = right - left;
        let height = bottom - top;
        let scale = (MAX_DISPLAY_WIDTH / width).min(MAX_DISPLAY_HEIGHT / height);
        let tx = cx - (left + frame_index as f32 * sprite.frame_width + width * 0.5) * scale;
        let ty = cy - (top + height * 0.5) * scale;
        let transform = Transform::from_row(scale, 0.0, 0.0, scale, tx, ty);

        for shape in frame {
            let mut paint = Paint::default();
            paint.anti_alias = true;
            paint.set_color_rgba8(
                (shape.color.red() * 255.0).round() as u8,
                (shape.color.green() * 255.0).round() as u8,
                (shape.color.blue() * 255.0).round() as u8,
                (alpha.clamp(0.0, 1.0) * shape.opacity * 255.0).round() as u8,
            );
            pixmap.fill_path(&shape.path, &paint, FillRule::EvenOdd, transform, None);
        }
    }
}

fn parse_svg(svg: &str) -> ExpressionSprite {
    let root_start = svg.find("<svg").expect("missing SVG root");
    let root_end = svg[root_start..]
        .find('>')
        .map(|offset| root_start + offset)
        .expect("invalid SVG root");
    let root = &svg[root_start..=root_end];
    let frame_width = attribute(root, "width")
        .parse::<f32>()
        .expect("invalid SVG width")
        / FRAME_COUNT as f32;
    let mut sprite = ExpressionSprite {
        frames: (0..FRAME_COUNT).map(|_| Vec::new()).collect(),
        bounds: None,
        frame_width,
    };
    for (path_start, _) in svg.match_indices("<path") {
        let tag_end = svg[path_start..]
            .find('>')
            .map(|i| path_start + i)
            .expect("valid SVG path tag");
        let tag = &svg[path_start..=tag_end];
        let frame = attribute(tag, "data-frame")
            .parse::<usize>()
            .expect("invalid SVG frame index");
        let color = parse_color(attribute(tag, "fill"));
        let opacity = optional_attribute(tag, "fill-opacity")
            .unwrap_or("1")
            .parse::<f32>()
            .expect("invalid SVG fill opacity");
        let mut tokens = attribute(tag, "d").split_whitespace();
        let mut builder = PathBuilder::new();
        while let Some(command) = tokens.next() {
            match command {
                "M" => builder.move_to(number(&mut tokens), number(&mut tokens)),
                "L" => builder.line_to(number(&mut tokens), number(&mut tokens)),
                "Q" => builder.quad_to(
                    number(&mut tokens),
                    number(&mut tokens),
                    number(&mut tokens),
                    number(&mut tokens),
                ),
                "Z" | "z" => builder.close(),
                other => panic!("unsupported SVG path command: {other}"),
            }
        }
        if let Some(path) = builder.finish() {
            sprite.frames[frame].push(ColoredPath {
                path,
                color,
                opacity,
            });
        }
    }
    let mut last_populated_frame: Option<Vec<ColoredPath>> = None;
    for frame in &mut sprite.frames {
        if frame.is_empty() {
            if let Some(previous) = &last_populated_frame {
                *frame = previous.clone();
            }
        } else {
            last_populated_frame = Some(frame.clone());
        }
    }
    sprite.bounds = sprite.frames.iter().enumerate().fold(
        None::<(f32, f32, f32, f32)>,
        |bounds, (frame_index, frame)| {
            frame
                .iter()
                .map(|shape| shape.path.bounds())
                .fold(bounds, |bounds, rect| {
                    let offset_x = frame_index as f32 * sprite.frame_width;
                    let (left, right) = (rect.left() - offset_x, rect.right() - offset_x);
                    Some(match bounds {
                        None => (left, rect.top(), right, rect.bottom()),
                        Some((l, t, r, b)) => (
                            l.min(left),
                            t.min(rect.top()),
                            r.max(right),
                            b.max(rect.bottom()),
                        ),
                    })
                })
        },
    );
    sprite
}

fn attribute<'a>(tag: &'a str, name: &str) -> &'a str {
    let key = format!("{name}=\"");
    let start = tag.find(&key).expect("missing SVG path attribute") + key.len();
    let end = tag[start..]
        .find('"')
        .map(|i| start + i)
        .expect("unterminated SVG attribute");
    &tag[start..end]
}

fn optional_attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!("{name}=\"");
    let start = tag.find(&key)? + key.len();
    let end = tag[start..].find('"').map(|i| start + i)?;
    Some(&tag[start..end])
}

fn number(tokens: &mut std::str::SplitWhitespace<'_>) -> f32 {
    tokens
        .next()
        .expect("missing SVG path coordinate")
        .parse()
        .expect("invalid SVG coordinate")
}

fn parse_color(value: &str) -> Color {
    let hex = value
        .strip_prefix('#')
        .expect("SVG fill must be a hex color");
    assert_eq!(hex.len(), 6, "SVG fill must be #RRGGBB");
    let channel =
        |start| u8::from_str_radix(&hex[start..start + 2], 16).expect("invalid SVG color");
    Color::from_rgba8(channel(0), channel(2), channel(4), 255)
}

fn expression_cell(expression: EmoExpression) -> usize {
    match expression {
        EmoExpression::Atlas(index) => index as usize,
        EmoExpression::Normal => 27,    // smile.svg
        EmoExpression::Relaxed => 28,   // smirk.svg
        EmoExpression::Winking => 19,   // nheo_mat.svg
        EmoExpression::Angry => 10,     // gian_du.svg
        EmoExpression::Rage => 6,       // demon_slayer.svg
        EmoExpression::Sleepy => 26,    // sleepy.svg
        EmoExpression::Sleeping => 26,  // sleepy.svg
        EmoExpression::AngryBlush => 7, // devil.svg
        EmoExpression::Crying => 3,     // cry.svg
        EmoExpression::FlatSlit => 19,  // nheo_mat.svg
        EmoExpression::Happy => 5,      // cuoi.svg
        EmoExpression::Squint => 29,    // squint.svg
        EmoExpression::Love => 34,      // UwU.svg
        EmoExpression::Sparkle => 24,   // rainbow.svg
        EmoExpression::Surprised => 17, // ngac_nhien.svg
        EmoExpression::Starry => 34,    // UwU.svg
        EmoExpression::Dizzy => 2,      // chong_mat.svg
    }
}
