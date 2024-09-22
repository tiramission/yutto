use crate::comment::{Comment, CommentPosition};
use crate::writer::rows;
use crate::writer::utils;

pub fn write_head(
    width: u32,
    height: u32,
    fontface: &str,
    fontsize: f32,
    alpha: f32,
    styleid: &str,
) -> String {
    let alpha = 255 - (alpha * 255.0).round() as u8;
    let outline = f32::max(fontsize / 25.0, 1.0);
    format!("\
[Script Info]
; Script generated by biliass (based on Danmaku2ASS)
; https://github.com/yutto-dev/yutto/tree/main/packages/biliass
Script Updated By: biliass (https://github.com/yutto-dev/yutto/tree/main/packages/biliass)
ScriptType: v4.00+
PlayResX: {width}
PlayResY: {height}
Aspect Ratio: {width}:{height}
Collisions: Normal
WrapStyle: 2
ScaledBorderAndShadow: yes
YCbCr Matrix: TV.601

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: {styleid}, {fontface}, {fontsize:.0}, &H{alpha:02X}FFFFFF, &H{alpha:02X}FFFFFF, &H{alpha:02X}000000, &H{alpha:02X}000000, 0, 0, 0, 0, 100, 100, 0.00, 0.00, 1, {outline:.0}, 0, 7, 0, 0, 0, 0

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"
    )
}

fn convert_type2(row: usize, height: u32, bottom_reserved: u32) -> usize {
    height as usize - bottom_reserved as usize - row
}

#[allow(clippy::too_many_arguments)]
pub fn write_comment(
    comment: &Comment,
    row: usize,
    width: u32,
    height: u32,
    bottom_reserved: u32,
    fontsize: f32,
    duration_marquee: f64,
    duration_still: f64,
    styleid: &str,
) -> String {
    let text = utils::ass_escape(&comment.comment);
    let (style, duration) = match comment.pos {
        CommentPosition::Bottom => {
            let halfwidth = width / 2;
            (format!("\\an8\\pos({halfwidth}, {row})"), duration_still)
        }
        CommentPosition::Top => {
            let halfwidth = width / 2;
            let row = convert_type2(row, height, bottom_reserved);
            (format!("\\an2\\pos({halfwidth}, {row})"), duration_still)
        }
        CommentPosition::Reversed => {
            let neglen = -(comment.width.ceil()) as i32;
            (
                format!("\\move({neglen}, {row}, {width}, {row})"),
                duration_marquee,
            )
        }
        _ => {
            let neglen = -(comment.width.ceil()) as i32;
            (
                format!("\\move({width}, {row}, {neglen}, {row})"),
                duration_marquee,
            )
        }
    };
    let mut styles = vec![style];
    if comment.size - fontsize <= -1. || comment.size - fontsize >= 1. {
        styles.push(format!("\\fs{:.0}", comment.size));
    }
    if comment.color != 0xFFFFFF {
        styles.push(format!(
            "\\c&H{}&",
            utils::convert_color(comment.color, None, None)
        ));
        if comment.color == 0x000000 {
            styles.push("\\3c&HFFFFFF&".to_owned());
        }
    }
    let start = utils::convert_timestamp(comment.timeline);
    let end = utils::convert_timestamp(comment.timeline + duration);
    let styles = styles.join("");
    format!("Dialogue: 2,{start},{end},{styleid},,0000,0000,0000,,{{{styles}}}{text}\n")
}

#[allow(clippy::too_many_arguments)]
pub fn write_normal_comment(
    rows: &mut rows::Rows,
    comment: &Comment,
    width: u32,
    height: u32,
    bottom_reserved: u32,
    fontsize: f32,
    duration_marquee: f64,
    duration_still: f64,
    styleid: &str,
    reduced: bool,
) -> String {
    let mut row: usize = 0;
    let rowmax = height - bottom_reserved - comment.height as u32;
    while row <= rowmax as usize {
        let freerows = rows::test_free_rows(
            rows,
            comment,
            row,
            width,
            height,
            bottom_reserved,
            duration_marquee,
            duration_still,
        );
        if freerows >= comment.height as usize {
            rows::mark_comment_row(rows, comment, row);
            return write_comment(
                comment,
                row,
                width,
                height,
                bottom_reserved,
                fontsize,
                duration_marquee,
                duration_still,
                styleid,
            );
        } else {
            row += if freerows == 0 { 1 } else { freerows };
        }
    }
    if !reduced {
        row = rows::find_alternative_row(rows, comment, height, bottom_reserved);
        rows::mark_comment_row(rows, comment, row);
        return write_comment(
            comment,
            row,
            width,
            height,
            bottom_reserved,
            fontsize,
            duration_marquee,
            duration_still,
            styleid,
        );
    }
    "".to_owned()
}

#[allow(clippy::too_many_arguments)]
pub fn write_comment_with_animation(
    comment: &Comment,
    width: u32,
    height: u32,
    rotate_y: f64,
    rotate_z: f64,
    from_x: f64,
    from_y: f64,
    to_x: f64,
    to_y: f64,
    from_alpha: u8,
    to_alpha: u8,
    text: &str,
    delay: f64,
    lifetime: f64,
    duration: f64,
    fontface: &str,
    is_border: bool,
    styleid: &str,
    zoom_factor: (f32, f32, f32),
) -> String {
    let from_rotarg = utils::convert_flash_rotation(
        rotate_y,
        rotate_z,
        from_x,
        from_y,
        width as f64,
        height as f64,
    );
    let to_rotarg =
        utils::convert_flash_rotation(rotate_y, rotate_z, to_x, to_y, width as f64, height as f64);
    if vec![
        from_rotarg.0,
        from_rotarg.1,
        from_rotarg.2,
        from_rotarg.3,
        from_rotarg.4,
        from_rotarg.5,
        to_rotarg.0,
        to_rotarg.1,
        to_rotarg.2,
        to_rotarg.3,
        to_rotarg.4,
        to_rotarg.5,
    ]
    .into_iter()
    .any(|x| x.is_nan())
    {
        // eprintln!(
        //     "Invalid rotation arguments: {:?}",
        //     (rotate_y, rotate_z, from_x, from_y)
        // );
        return "".to_owned();
    }
    let mut styles = vec![format!("\\org({}, {})", width / 2, height / 2)];
    if (from_rotarg.0, from_rotarg.1) == (to_rotarg.0, to_rotarg.1) {
        styles.push(format!("\\pos({:.0}, {:.0})", from_rotarg.0, from_rotarg.1));
    } else {
        styles.push(format!(
            "\\move({:.0}, {:.0}, {:.0}, {:.0}, {:.0}, {:.0})",
            from_rotarg.0,
            from_rotarg.1,
            to_rotarg.0,
            to_rotarg.1,
            delay,
            delay + duration
        ));
    }
    styles.push(format!(
        "\\frx{:.0}\\fry{:.0}\\frz{:.0}\\fscx{:.0}\\fscy{:.0}",
        from_rotarg.2, from_rotarg.3, from_rotarg.4, from_rotarg.5, from_rotarg.6
    ));
    if (from_x, from_y) != (to_x, to_y) {
        styles.push(format!(
            "\\t({}, {}, ",
            delay as i32,
            (delay + duration) as i32
        ));
        styles.push(format!(
            "\\frx{:.0}\\fry{:.0}\\frz{:.0}\\fscx{:.0}\\fscy{:.0}",
            to_rotarg.2, to_rotarg.3, to_rotarg.4, to_rotarg.5, to_rotarg.6
        ));
        styles.push(")".to_owned());
    }
    if !fontface.is_empty() {
        styles.push(format!("\\fn{}", utils::ass_escape(fontface)));
    }
    styles.push(format!("\\fs{:.0}", comment.size * zoom_factor.0));
    if comment.color != 0xFFFFFF {
        styles.push(format!(
            "\\c&H{}&",
            utils::convert_color(comment.color, None, None)
        ));
        if comment.color == 0x000000 {
            styles.push("\\3c&HFFFFFF&".to_owned());
        }
    }
    if from_alpha == to_alpha {
        styles.push(format!("\\alpha&H{from_alpha:02X}"));
    } else if (from_alpha, to_alpha) == (255, 0) {
        styles.push(format!("\\fad({:.0},0)", lifetime * 1000.))
    } else if (from_alpha, to_alpha) == (0, 255) {
        styles.push(format!("\\fad(0, {:.0})", lifetime * 1000.));
    } else {
        let lifetime = lifetime * 1000.;
        styles.push(
            format!(
                "\\fade({from_alpha}, {to_alpha}, {to_alpha}, 0, {lifetime:.0}, {lifetime:.0}, {lifetime:.0})"
            )
        )
    }
    if !is_border {
        styles.push("\\bord0".to_owned())
    }
    let start = utils::convert_timestamp(comment.timeline);
    let end = utils::convert_timestamp(comment.timeline + lifetime);
    let styles = styles.join("");
    let text = utils::ass_escape(text);
    format!("Dialogue: -1,{start},{end},{styleid},,0,0,0,,{{{styles}}}{text}\n")
}