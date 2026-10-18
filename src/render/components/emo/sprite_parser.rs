//! Parser for the bundled SVG expression strips.

use tiny_skia::{Color, Path, PathBuilder};

pub(super) const EXPRESSION_COUNT: usize = 35;
pub(super) const FRAME_COUNT: usize = 24;
pub(super) const FRAME_RATE: f32 = 6.0;
pub(super) const EXPRESSION_DURATION: f32 = FRAME_COUNT as f32 / FRAME_RATE;

pub(super) struct ExpressionSprite {
    pub(super) frames: Vec<Vec<ColoredPath>>,
    pub(super) bounds: Option<(f32, f32, f32, f32)>,
    pub(super) frame_width: f32,
}

#[derive(Clone)]
pub(super) struct ColoredPath {
    pub(super) path: Path,
    pub(super) color: Color,
    pub(super) opacity: f32,
}

pub(super) const EXPRESSION_SVGS: [&str; EXPRESSION_COUNT] = [
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

pub(super) fn parse_svg(svg: &str) -> ExpressionSprite {
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
            .map(|offset| path_start + offset)
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
                        Some((left_bound, top, right_bound, bottom)) => (
                            left_bound.min(left),
                            top.min(rect.top()),
                            right_bound.max(right),
                            bottom.max(rect.bottom()),
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
        .map(|offset| start + offset)
        .expect("unterminated SVG attribute");
    &tag[start..end]
}

fn optional_attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!("{name}=\"");
    let start = tag.find(&key)? + key.len();
    let end = tag[start..].find('"').map(|offset| start + offset)?;
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
