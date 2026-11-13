use image::RgbaImage;

const CHAR_ASPECT: f32 = 0.5;
const VISOR_RAMP: [[u8; 3]; 6] = [
    [0x59, 0x2a, 0xa4],
    [0x42, 0x77, 0xa2],
    [0x3f, 0xb6, 0xaf],
    [0x93, 0x9e, 0x7a],
    [0xc1, 0x8d, 0x5d],
    [0xd9, 0x63, 0x50],
];
const CREAM_RGB: [u8; 3] = [0xf4, 0xf3, 0xe8];
const AMBER_RGB: [u8; 3] = [0xc1, 0x8d, 0x5d];
const AMBER_LO: [u8; 3] = [0x18, 0x0e, 0x06];

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
}

impl Look {
    pub const ALL: [Look; 4] = [Look::Visor, Look::Blocks, Look::Crt, Look::Ansi];

    pub fn name(self) -> &'static str {
        match self {
            Self::Visor => "VISOR",
            Self::Blocks => "BLOCKS",
            Self::Crt => "CRT",
            Self::Ansi => "ANSI",
        }
    }

    pub fn knob(self) -> (&'static str, std::ops::RangeInclusive<f32>) {
        match self {
            Self::Visor | Self::Blocks => ("weight", 0.4..=2.4),
            Self::Crt => ("glow", -0.5..=0.5),
            Self::Ansi => ("chroma", 0.0..=2.0),
        }
    }

    pub fn apply(self, s: &mut Settings) {
        s.charset = match self {
            Self::Visor | Self::Blocks => Charset::Blocks,
            Self::Crt => Charset::Punctuation,
            Self::Ansi => Charset::Alphanumeric,
        };
        s.color_kind = match self {
            Self::Visor => ColorKind::Visor,
            Self::Crt => ColorKind::Amber,
            Self::Blocks | Self::Ansi => ColorKind::Source,
        };
        s.color = !matches!(self, Self::Crt);
        match self {
            Self::Visor | Self::Blocks => {
                s.saturation = 1.0;
                s.brightness = 0.0;
            }
            Self::Crt => {
                s.contrast = 1.0;
                s.saturation = 0.0;
            }
            Self::Ansi => {
                s.contrast = 1.0;
                s.saturation = 1.0;
                s.brightness = 0.0;
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct Settings {
    pub cols: u32,
    pub charset: Charset,
    pub mark: Mark,
    pub contrast: f32,
    pub saturation: f32,
    pub brightness: f32,
    pub invert: bool,
    pub color: bool,
    pub color_kind: ColorKind,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            cols: 80,
            charset: Charset::Blocks,
            mark: Mark::Cells,
            contrast: 1.0,
            saturation: 1.0,
            brightness: 0.0,
            invert: false,
            color: true,
            color_kind: ColorKind::Source,
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
            let rgb = sample(img, x, y, cell_w, cell_h);
            let rgb = enhance(rgb, s);
            let mut yv = luma(rgb);
            if s.invert {
                yv = 1.0 - yv;
            }
            let ch = ramp_char(ramp, yv);
            let color = cell_color(s, rgb, yv);
            out.push(Cell { ch, color });
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
        ColorKind::Amber => amber_rgb(yv),
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

fn amber_rgb(y: f32) -> [u8; 3] {
    let y = y.clamp(0.0, 1.0);
    if y > 0.85 {
        lerp_u8(AMBER_RGB, CREAM_RGB, (y - 0.85) / 0.15)
    } else {
        lerp_u8(AMBER_LO, AMBER_RGB, y / 0.85)
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
}
