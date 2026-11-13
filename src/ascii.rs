use image::RgbaImage;

pub const CHAR_ASPECT: f32 = 0.5;
/// Noto Sans Mono advance / em. Glyphs sized to cell height are wider than
/// a 1:2 cell, so high-col CHARS overlapped and moiré-hatched.
pub const MONO_ADVANCE: f32 = 0.6;
const VISOR_RAMP: [[u8; 3]; 6] = [
    [0x59, 0x2a, 0xa4],
    [0x42, 0x77, 0xa2],
    [0x3f, 0xb6, 0xaf],
    [0x93, 0x9e, 0x7a],
    [0xc1, 0x8d, 0x5d],
    [0xd9, 0x63, 0x50],
];
const CREAM_RGB: [u8; 3] = [0xf4, 0xf3, 0xe8];
const GROUND_RGB: [u8; 3] = [0x05, 0x05, 0x08];
const CHARCOAL_RGB: [u8; 3] = [0x24, 0x2a, 0x30];
const SAGE_RGB: [u8; 3] = [0x93, 0x9e, 0x7a];
const TEAL_RGB: [u8; 3] = [0x3f, 0xb6, 0xaf];
const CORAL_RGB: [u8; 3] = [0xd9, 0x63, 0x50];
const AMBER_RGB: [u8; 3] = [0xc1, 0x8d, 0x5d];
const AMBER_LO: [u8; 3] = [0x18, 0x0e, 0x06];
const TEAL_LO: [u8; 3] = [0x06, 0x14, 0x14];
const CORAL_LO: [u8; 3] = [0x16, 0x08, 0x08];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Charset {
    Alphanumeric,
    Blocks,
    Punctuation,
}

impl Charset {
    pub fn ramp(self) -> &'static [char] {
        match self {
            Self::Alphanumeric => &[
                ' ', '.', '\'', '`', '^', '"', ':', ';', 'I', 'l', '!', 'i', '~', '+', '_', '-',
                '?', '1', '|', '(', ')', 't', 'f', 'j', 'r', 'x', 'n', 'u', 'v', 'c', 'z', 'X',
                'Y', 'U', 'J', 'C', 'L', 'Q', '0', 'O', 'Z', 'm', 'w', 'q', 'p', 'd', 'b', 'k',
                'h', 'a', 'o', '*', '#', 'M', 'W', '&', '8', '%', 'B', '@', '$',
            ],
            Self::Blocks => &[' ', '░', '▒', '▓', '█'],
            Self::Punctuation => &[' ', '.', ',', ':', ';', '!', '?', '-', '+', '=', '*', '#'],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ColorKind {
    Source,
    Visor,
    Amber,
    Cream,
    Teal,
    Coral,
    Sage,
    Duo,
}

impl ColorKind {
    pub const ALL: [ColorKind; 8] = [
        ColorKind::Source,
        ColorKind::Visor,
        ColorKind::Amber,
        ColorKind::Cream,
        ColorKind::Teal,
        ColorKind::Coral,
        ColorKind::Sage,
        ColorKind::Duo,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Source => "SOURCE",
            Self::Visor => "VISOR",
            Self::Amber => "AMBER",
            Self::Cream => "CREAM",
            Self::Teal => "TEAL",
            Self::Coral => "CORAL",
            Self::Sage => "SAGE",
            Self::Duo => "DUO",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Cells,
    Chars,
}

impl Mark {
    pub fn name(self) -> &'static str {
        match self {
            Self::Cells => "CELLS",
            Self::Chars => "CHARS",
        }
    }

    pub fn aspect(self) -> f32 {
        match self {
            Self::Cells => 1.0,
            Self::Chars => CHAR_ASPECT,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Look {
    Visor,
    Blocks,
    Crt,
    Ansi,
    Dither,
    Hatch,
    Scan,
}

impl Look {
    pub const ALL: [Look; 7] = [
        Look::Visor,
        Look::Blocks,
        Look::Crt,
        Look::Ansi,
        Look::Dither,
        Look::Hatch,
        Look::Scan,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Visor => "VISOR",
            Self::Blocks => "BLOCKS",
            Self::Crt => "CRT",
            Self::Ansi => "ANSI",
            Self::Dither => "DITHER",
            Self::Hatch => "HATCH",
            Self::Scan => "SCAN",
        }
    }

    pub fn knob(self) -> (&'static str, std::ops::RangeInclusive<f32>) {
        match self {
            Self::Visor | Self::Blocks | Self::Dither | Self::Hatch => ("weight", 0.4..=4.0),
            Self::Crt | Self::Scan => ("glow", -0.5..=0.5),
            Self::Ansi => ("chroma", 0.0..=2.0),
        }
    }

    pub fn apply(self, s: &mut Settings) {
        s.look = self;
        s.charset = match self {
            Self::Visor | Self::Blocks | Self::Dither | Self::Hatch | Self::Scan => Charset::Blocks,
            Self::Crt => Charset::Punctuation,
            Self::Ansi => Charset::Alphanumeric,
        };
        s.color_kind = match self {
            Self::Visor => ColorKind::Visor,
            Self::Crt => ColorKind::Amber,
            Self::Blocks | Self::Ansi | Self::Dither => ColorKind::Source,
            Self::Hatch => ColorKind::Sage,
            Self::Scan => ColorKind::Teal,
        };
        s.color = !matches!(self, Self::Crt);
        s.contrast = 1.0;
        s.saturation = 1.0;
        s.brightness = 0.0;
        if matches!(self, Self::Crt) {
            s.saturation = 0.0;
        }
    }
}

#[derive(Clone, Copy)]
pub struct Settings {
    pub cols: u32,
    pub charset: Charset,
    pub mark: Mark,
    pub look: Look,
    pub contrast: f32,
    pub saturation: f32,
    pub brightness: f32,
    pub invert: bool,
    pub color: bool,
    pub color_kind: ColorKind,
    pub duo_ink: [u8; 3],
    pub duo_paper: [u8; 3],
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            cols: 80,
            charset: Charset::Blocks,
            mark: Mark::Cells,
            look: Look::Visor,
            contrast: 1.0,
            saturation: 1.0,
            brightness: 0.0,
            invert: false,
            color: true,
            color_kind: ColorKind::Visor,
            duo_ink: CHARCOAL_RGB,
            duo_paper: CREAM_RGB,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Cell {
    pub ch: char,
    pub color: [u8; 3],
}

pub fn convert(img: &RgbaImage, s: &Settings) -> (u32, u32, Vec<Cell>) {
    let cols = s.cols.clamp(8, 400);
    let img_aspect = img.width().max(1) as f32 / img.height().max(1) as f32;
    let rows = ((cols as f32 / img_aspect) * s.mark.aspect()).round().max(1.0) as u32;
    let cell_w = img.width() as f32 / cols as f32;
    let cell_h = img.height() as f32 / rows as f32;
    let ramp = s.charset.ramp();
    let mut out = Vec::with_capacity((cols * rows) as usize);
    for y in 0..rows {
        for x in 0..cols {
            let rgb = enhance(sample(img, x, y, cell_w, cell_h), s);
            let mut yv = luma(rgb);
            if s.invert {
                yv = 1.0 - yv;
            }
            let ch = glyph(s, ramp, x, y, yv);
            out.push(Cell {
                ch,
                color: cell_color(s, rgb, yv),
            });
        }
    }
    (cols, rows, out)
}

pub fn ramp_char(ramp: &[char], luma: f32) -> char {
    if ramp.is_empty() {
        return ' ';
    }
    let t = luma.clamp(0.0, 1.0);
    let i = ((t * ramp.len() as f32) as usize).min(ramp.len() - 1);
    ramp[i]
}

fn glyph(s: &Settings, ramp: &[char], x: u32, y: u32, yv: f32) -> char {
    match s.look {
        Look::Dither => {
            if yv > bayer4(x, y) {
                '█'
            } else {
                ' '
            }
        }
        Look::Hatch => {
            let period = 5u32;
            let slot = (x + y) % period;
            if yv * period as f32 > slot as f32 {
                '█'
            } else {
                ' '
            }
        }
        Look::Scan => {
            if y % 2 == 1 {
                ' '
            } else {
                ramp_char(ramp, yv)
            }
        }
        _ => ramp_char(ramp, yv),
    }
}

fn bayer4(x: u32, y: u32) -> f32 {
    const M: [[f32; 4]; 4] = [
        [0.0 / 16.0, 8.0 / 16.0, 2.0 / 16.0, 10.0 / 16.0],
        [12.0 / 16.0, 4.0 / 16.0, 14.0 / 16.0, 6.0 / 16.0],
        [3.0 / 16.0, 11.0 / 16.0, 1.0 / 16.0, 9.0 / 16.0],
        [15.0 / 16.0, 7.0 / 16.0, 13.0 / 16.0, 5.0 / 16.0],
    ];
    M[(y % 4) as usize][(x % 4) as usize]
}

fn sample(img: &RgbaImage, cx: u32, cy: u32, cw: f32, ch: f32) -> [f32; 3] {
    let x0 = (cx as f32 * cw) as u32;
    let y0 = (cy as f32 * ch) as u32;
    let x1 = (((cx + 1) as f32 * cw) as u32).min(img.width()).max(x0 + 1);
    let y1 = (((cy + 1) as f32 * ch) as u32).min(img.height()).max(y0 + 1);
    let sx = ((x1 - x0) / 4).max(1);
    let sy = ((y1 - y0) / 4).max(1);
    let mut acc = [0.0f32; 3];
    let mut n = 0.0;
    let mut y = y0;
    while y < y1 {
        let mut x = x0;
        while x < x1 {
            let p = img.get_pixel(x, y).0;
            let a = p[3] as f32 / 255.0;
            acc[0] += p[0] as f32 / 255.0 * a;
            acc[1] += p[1] as f32 / 255.0 * a;
            acc[2] += p[2] as f32 / 255.0 * a;
            n += 1.0;
            x += sx;
        }
        y += sy;
    }
    if n == 0.0 {
        return [0.0; 3];
    }
    [acc[0] / n, acc[1] / n, acc[2] / n]
}

fn enhance(rgb: [f32; 3], s: &Settings) -> [f32; 3] {
    let y = luma(rgb);
    let sat = s.saturation;
    let mut c = [
        y + (rgb[0] - y) * sat,
        y + (rgb[1] - y) * sat,
        y + (rgb[2] - y) * sat,
    ];
    c = c.map(|v| (v - 0.5) * s.contrast + 0.5 + s.brightness);
    c.map(|v| v.clamp(0.0, 1.0))
}

fn luma(rgb: [f32; 3]) -> f32 {
    0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}

fn to_u8(rgb: [f32; 3]) -> [u8; 3] {
    rgb.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
}

fn cell_color(s: &Settings, rgb: [f32; 3], yv: f32) -> [u8; 3] {
    match s.color_kind {
        ColorKind::Visor => visor_rgb(yv),
        ColorKind::Amber => phosphor(AMBER_LO, AMBER_RGB, CREAM_RGB, yv),
        ColorKind::Teal => phosphor(TEAL_LO, TEAL_RGB, CREAM_RGB, yv),
        ColorKind::Coral => phosphor(CORAL_LO, CORAL_RGB, CREAM_RGB, yv),
        ColorKind::Cream => lerp_u8(GROUND_RGB, CREAM_RGB, yv.clamp(0.0, 1.0)),
        ColorKind::Sage => {
            let y = yv.clamp(0.0, 1.0);
            if y < 0.5 {
                lerp_u8(GROUND_RGB, SAGE_RGB, y * 2.0)
            } else {
                lerp_u8(SAGE_RGB, CREAM_RGB, (y - 0.5) * 2.0)
            }
        }
        ColorKind::Duo => lerp_u8(s.duo_ink, s.duo_paper, yv.clamp(0.0, 1.0)),
        ColorKind::Source if s.color => to_u8(rgb),
        ColorKind::Source => {
            let g = (yv * 255.0).round() as u8;
            [g, g, g]
        }
    }
}

fn visor_rgb(y: f32) -> [u8; 3] {
    let n = VISOR_RAMP.len();
    let t = y.clamp(0.0, 1.0) * (n as f32 - 1.0);
    let i = (t.floor() as usize).min(n - 1);
    let f = t - i as f32;
    let a = VISOR_RAMP[i];
    let b = VISOR_RAMP[(i + 1).min(n - 1)];
    let mixed = lerp_u8(a, b, f);
    lerp_u8(mixed, CREAM_RGB, y.clamp(0.0, 1.0) * 0.22)
}

fn phosphor(lo: [u8; 3], mid: [u8; 3], hi: [u8; 3], y: f32) -> [u8; 3] {
    let y = y.clamp(0.0, 1.0);
    if y > 0.85 {
        lerp_u8(mid, hi, (y - 0.85) / 0.15)
    } else {
        lerp_u8(lo, mid, y / 0.85)
    }
}

fn lerp_u8(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t).round() as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t).round() as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t).round() as u8,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ramp_maps_ends() {
        let r = Charset::Blocks.ramp();
        assert_eq!(ramp_char(r, 0.0), ' ');
        assert_eq!(ramp_char(r, 1.0), '█');
    }

    #[test]
    fn convert_grid_size() {
        let img = RgbaImage::from_pixel(100, 50, image::Rgba([255, 255, 255, 255]));
        let mut s = Settings::default();
        s.cols = 20;
        s.charset = Charset::Blocks;
        s.mark = Mark::Chars;
        let (cols, rows, cells) = convert(&img, &s);
        assert_eq!(cols, 20);
        assert_eq!(rows, 5);
        assert_eq!(cells.len(), 100);
        assert!(cells.iter().all(|c| c.ch == '█'));
    }

    #[test]
    fn convert_cells_are_square() {
        let img = RgbaImage::from_pixel(100, 50, image::Rgba([255, 255, 255, 255]));
        let mut s = Settings::default();
        s.cols = 20;
        s.mark = Mark::Cells;
        let (cols, rows, cells) = convert(&img, &s);
        assert_eq!(cols, 20);
        assert_eq!(rows, 10);
        assert_eq!(cells.len(), 200);
    }

    #[test]
    fn convert_invert_maps_white_to_space() {
        let img = RgbaImage::from_pixel(40, 40, image::Rgba([255, 255, 255, 255]));
        let mut s = Settings::default();
        s.cols = 8;
        s.charset = Charset::Blocks;
        s.invert = true;
        s.color = false;
        let (_, _, cells) = convert(&img, &s);
        assert!(cells.iter().all(|c| c.ch == ' '));
    }

    #[test]
    fn visor_and_crt_use_named_palettes() {
        let img = RgbaImage::from_pixel(40, 40, image::Rgba([255, 255, 255, 255]));
        let mut visor = Settings::default();
        visor.cols = 8;
        Look::Visor.apply(&mut visor);
        let (_, _, cells) = convert(&img, &visor);
        assert!(cells.iter().all(|c| c.ch == '█'));
        assert!(cells.iter().any(|c| c.color != [255, 255, 255]));

        let mut crt = Settings::default();
        crt.cols = 8;
        Look::Crt.apply(&mut crt);
        let (_, _, cells) = convert(&img, &crt);
        assert!(cells.iter().all(|c| c.color[0] >= c.color[2]));
    }

    #[test]
    fn ansi_after_crt_restores_chroma() {
        let img = RgbaImage::from_pixel(40, 40, image::Rgba([255, 0, 0, 255]));
        let mut s = Settings::default();
        s.cols = 8;
        Look::Crt.apply(&mut s);
        Look::Ansi.apply(&mut s);
        assert!((s.saturation - 1.0).abs() < f32::EPSILON);
        let (_, _, cells) = convert(&img, &s);
        assert!(cells.iter().any(|c| c.color[0] > c.color[1] && c.color[0] > c.color[2]));
    }

    #[test]
    fn cream_maps_white_to_cream() {
        let img = RgbaImage::from_pixel(20, 20, image::Rgba([255, 255, 255, 255]));
        let mut s = Settings::default();
        s.cols = 8;
        s.color_kind = ColorKind::Cream;
        let (_, _, cells) = convert(&img, &s);
        assert!(cells.iter().all(|c| c.color == CREAM_RGB));
    }

    #[test]
    fn duotone_maps_black_to_ink_white_to_paper() {
        let black = RgbaImage::from_pixel(20, 20, image::Rgba([0, 0, 0, 255]));
        let white = RgbaImage::from_pixel(20, 20, image::Rgba([255, 255, 255, 255]));
        let mut s = Settings::default();
        s.cols = 8;
        s.color_kind = ColorKind::Duo;
        s.duo_ink = [0xd9, 0x63, 0x50];
        s.duo_paper = CREAM_RGB;
        let (_, _, dark) = convert(&black, &s);
        let (_, _, light) = convert(&white, &s);
        assert!(dark.iter().all(|c| c.color == s.duo_ink));
        assert!(light.iter().all(|c| c.color == s.duo_paper));
        Look::Blocks.apply(&mut s);
        s.color_kind = ColorKind::Duo;
        let (_, _, cells) = convert(&white, &s);
        let ramp = s.charset.ramp();
        assert!(cells.iter().all(|c| ramp.contains(&c.ch)));
        assert!(cells.iter().all(|c| c.color == s.duo_paper));
    }

    #[test]
    fn duotone_accepts_arbitrary_rgb() {
        let black = RgbaImage::from_pixel(20, 20, image::Rgba([0, 0, 0, 255]));
        let mut s = Settings::default();
        s.cols = 8;
        s.color_kind = ColorKind::Duo;
        s.duo_ink = [0x12, 0xc8, 0x28];
        s.duo_paper = [0xee, 0x11, 0x99];
        let (_, _, dark) = convert(&black, &s);
        assert!(dark.iter().all(|c| c.color == s.duo_ink));
    }

    #[test]
    fn source_keeps_still_colour() {
        let img = RgbaImage::from_pixel(20, 20, image::Rgba([220, 40, 30, 255]));
        let mut s = Settings::default();
        s.cols = 8;
        Look::Visor.apply(&mut s);
        s.color_kind = ColorKind::Source;
        let (_, _, cells) = convert(&img, &s);
        assert!(cells.iter().any(|c| c.color[0] > c.color[1] && c.color[0] > c.color[2]));
    }

    #[test]
    fn dither_is_binary_occupancy() {
        let img = RgbaImage::from_pixel(40, 40, image::Rgba([128, 128, 128, 255]));
        let mut s = Settings::default();
        s.cols = 16;
        Look::Dither.apply(&mut s);
        let (_, _, cells) = convert(&img, &s);
        assert!(cells.iter().any(|c| c.ch == ' '));
        assert!(cells.iter().any(|c| c.ch == '█'));
        assert!(cells.iter().all(|c| c.ch == ' ' || c.ch == '█'));
    }

    #[test]
    fn hatch_is_diagonal_occupancy() {
        let img = RgbaImage::from_pixel(40, 40, image::Rgba([200, 200, 200, 255]));
        let mut s = Settings::default();
        s.cols = 16;
        Look::Hatch.apply(&mut s);
        let (cols, _, cells) = convert(&img, &s);
        assert!(cells.iter().any(|c| c.ch == ' '));
        assert!(cells.iter().any(|c| c.ch == '█'));
        let mut diagonal = 0;
        for (i, cell) in cells.iter().enumerate() {
            if cell.ch == '█' {
                let x = i as u32 % cols;
                let y = i as u32 / cols;
                if (x + y) % 5 == 0 {
                    diagonal += 1;
                }
            }
        }
        assert!(diagonal > 0);
    }

    #[test]
    fn scan_blanks_odd_rows() {
        let img = RgbaImage::from_pixel(40, 40, image::Rgba([255, 255, 255, 255]));
        let mut s = Settings::default();
        s.cols = 8;
        Look::Scan.apply(&mut s);
        let (cols, rows, cells) = convert(&img, &s);
        for y in 0..rows {
            let row = &cells[(y * cols) as usize..((y + 1) * cols) as usize];
            if y % 2 == 1 {
                assert!(row.iter().all(|c| c.ch == ' '));
            } else {
                assert!(row.iter().any(|c| c.ch != ' '));
            }
        }
    }

    #[test]
    fn teal_and_coral_highlights_are_cream() {
        let img = RgbaImage::from_pixel(20, 20, image::Rgba([255, 255, 255, 255]));
        for kind in [ColorKind::Teal, ColorKind::Coral, ColorKind::Sage] {
            let mut s = Settings::default();
            s.cols = 8;
            s.color_kind = kind;
            let (_, _, cells) = convert(&img, &s);
            assert!(cells.iter().all(|c| c.color == CREAM_RGB), "{:?} {:?}", kind.name(), cells[0].color);
        }
    }

}
