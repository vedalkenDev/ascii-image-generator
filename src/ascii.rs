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
    pub fn name(self) -> &'static str {
        match self {
            Self::Source => "Source",
            Self::Visor => "Spectrum",
            Self::Amber => "Amber",
            Self::Cream => "Cream",
            Self::Teal => "Teal",
            Self::Coral => "Coral",
            Self::Sage => "Sage",
            Self::Duo => "Duo",
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
            Self::Cells => "Cells",
            Self::Chars => "Chars",
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
            Self::Visor => "Visor",
            Self::Blocks => "Blocks",
            Self::Crt => "Crt",
            Self::Ansi => "Ansi",
            Self::Dither => "Dither",
            Self::Hatch => "Hatch",
            Self::Scan => "Scan",
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

#[derive(Clone, Copy, PartialEq)]
pub struct Settings {
    pub cols: u32,
    pub charset: Charset,
    pub mark: Mark,
    pub shade_cells: bool,
    pub visor_texture: f32,
    pub look: Look,
    pub contrast: f32,
    pub saturation: f32,
    pub brightness: f32,
    pub tone_brightness: f32,
    pub tone_contrast: f32,
    pub tone_gamma: f32,
    pub auto_levels: bool,
    pub invert: bool,
    pub color: bool,
    pub color_kind: ColorKind,
    pub filter_strength: f32,
    pub spectrum_shift: f32,
    pub spectrum_range: f32,
    pub spectrum_mix: f32,
    pub duo_ink: [u8; 3],
    pub duo_paper: [u8; 3],
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            cols: 80,
            charset: Charset::Blocks,
            mark: Mark::Cells,
            shade_cells: false,
            visor_texture: 0.65,
            look: Look::Visor,
            contrast: 1.0,
            saturation: 1.0,
            brightness: 0.0,
            tone_brightness: 0.0,
            tone_contrast: 1.0,
            tone_gamma: 1.0,
            auto_levels: false,
            invert: false,
            color: true,
            color_kind: ColorKind::Visor,
            filter_strength: 0.0,
            spectrum_shift: 0.0,
            spectrum_range: 1.0,
            spectrum_mix: 0.0,
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

pub struct Samples {
    pub cols: u32,
    pub rows: u32,
    pub rgb: Vec<[f32; 3]>,
}

pub fn sample_image(img: &RgbaImage, s: &Settings) -> Samples {
    let cols = s.cols.clamp(8, 400);
    let img_aspect = img.width().max(1) as f32 / img.height().max(1) as f32;
    let rows = ((cols as f32 / img_aspect) * s.mark.aspect()).round().max(1.0) as u32;
    let cell_w = img.width() as f32 / cols as f32;
    let cell_h = img.height() as f32 / rows as f32;
    let mut rgb = Vec::with_capacity((cols as usize) * (rows as usize));
    for y in 0..rows {
        for x in 0..cols {
            rgb.push(sample(img, x, y, cell_w, cell_h));
        }
    }
    Samples { cols, rows, rgb }
}

pub fn convert_samples(samples: &Samples, s: &Settings) -> (u32, u32, Vec<Cell>) {
    let ramp = s.charset.ramp();
    let levels = if s.auto_levels { sample_levels(samples) } else { None };
    let mut out = Vec::with_capacity(samples.rgb.len());
    for (i, &sampled) in samples.rgb.iter().enumerate() {
        let x = i as u32 % samples.cols;
        let y = i as u32 / samples.cols;
        let sampled = if let Some((low, high)) = levels {
            sampled.map(|v| ((v - low) / (high - low)).clamp(0.0, 1.0))
        } else {
            sampled
        };
        let rgb = adjust_tone(enhance(sampled, s), s);
        let mut yv = luma(rgb);
        if s.invert {
            yv = 1.0 - yv;
        }
        let ch = glyph(s, ramp, x, y, yv);
        let mut color = cell_color(s, rgb, yv);
        if s.mark == Mark::Cells && s.shade_cells && ch != ' ' {
            let coverage = ramp.iter().position(|&value| value == ch)
                .map(|index| index as f32 / (ramp.len().saturating_sub(1).max(1) as f32))
                .unwrap_or(1.0);
            color = lerp_u8(GROUND_RGB, color, coverage);
        }
        out.push(Cell { ch, color });
    }
    (samples.cols, samples.rows, out)
}

fn sample_levels(samples: &Samples) -> Option<(f32, f32)> {
    if samples.rgb.is_empty() {
        return None;
    }
    let mut histogram = [0usize; 256];
    for &rgb in &samples.rgb {
        histogram[(luma(rgb).clamp(0.0, 1.0) * 255.0).round() as usize] += 1;
    }
    let n = samples.rgb.len() - 1;
    let percentile = |rank: usize| {
        let mut count = 0;
        for (level, frequency) in histogram.iter().enumerate() {
            count += frequency;
            if count > rank { return level; }
        }
        255
    };
    let low = percentile((n * 2 + 50) / 100);
    let high = percentile((n * 98 + 50) / 100);
    if high <= low + 1 { return None; }
    Some((low as f32 / 255.0, high as f32 / 255.0))
}

pub fn convert(img: &RgbaImage, s: &Settings) -> (u32, u32, Vec<Cell>) {
    convert_samples(&sample_image(img, s), s)
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
        Look::Visor if s.visor_texture > 0.0 => {
            let amount = s.visor_texture.clamp(0.0, 1.0);
            // The envelope keeps pure black and white stable while adding
            // ordered texture to the tones between them.
            let envelope = 4.0 * yv.clamp(0.0, 1.0) * (1.0 - yv.clamp(0.0, 1.0));
            let offset = (bayer4(x, y) - 7.5 / 16.0) * 0.9 * amount * envelope;
            ramp_char(ramp, yv + offset)
        }
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

fn adjust_tone(rgb: [f32; 3], s: &Settings) -> [f32; 3] {
    // Preserve the old pipeline exactly at the default settings.
    if s.tone_brightness == 0.0 && s.tone_contrast == 1.0 && s.tone_gamma == 1.0 {
        return rgb;
    }
    let gamma = s.tone_gamma.clamp(0.1, 4.0);
    rgb.map(|v| {
        let level = ((v - 0.5) * s.tone_contrast + 0.5 + s.tone_brightness)
            .clamp(0.0, 1.0);
        level.powf(1.0 / gamma)
    })
}

fn luma(rgb: [f32; 3]) -> f32 {
    0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}

fn to_u8(rgb: [f32; 3]) -> [u8; 3] {
    rgb.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
}

fn cell_color(s: &Settings, rgb: [f32; 3], yv: f32) -> [u8; 3] {
    let base = match s.color_kind {
        ColorKind::Visor => {
            let position = if s.spectrum_shift == 0.0 && s.spectrum_range == 1.0 {
                yv
            } else {
                ((yv - 0.5) * s.spectrum_range.clamp(0.0, 2.0) + 0.5 + s.spectrum_shift)
                    .clamp(0.0, 1.0)
            };
            let spectrum = visor_rgb(position);
            if s.spectrum_mix == 0.0 { spectrum } else {
                lerp_u8(spectrum, to_u8(rgb), s.spectrum_mix)
            }
        },
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
    };
    let strength = s.filter_strength.clamp(0.0, 1.0);
    if strength == 0.0 { return base; }
    let y = yv.clamp(0.0, 1.0);
    let enhanced = match s.color_kind {
        ColorKind::Cream => lerp_u8(GROUND_RGB, CREAM_RGB, y.sqrt()),
        ColorKind::Teal => lerp_u8(TEAL_LO, TEAL_RGB, y),
        ColorKind::Coral => lerp_u8(CORAL_LO, CORAL_RGB, y),
        _ => return base,
    };
    lerp_u8(base, enhanced, strength)
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
    fn spectrum_controls_change_colours_without_changing_glyphs() {
        let img = RgbaImage::from_pixel(32, 32, image::Rgba([180, 100, 40, 255]));
        let mut s = Settings::default();
        s.cols = 8;
        let (_, _, baseline) = convert(&img, &s);
        s.spectrum_shift = 0.3;
        let (_, _, shifted) = convert(&img, &s);
        assert_ne!(baseline[0].color, shifted[0].color);
        assert!(baseline.iter().zip(&shifted).all(|(a, b)| a.ch == b.ch));
        s.spectrum_shift = 0.0;
        s.spectrum_range = 0.0;
        let (_, _, narrowed) = convert(&img, &s);
        assert_ne!(baseline[0].color, narrowed[0].color);
        s.spectrum_range = 1.0;
        s.spectrum_mix = 1.0;
        let (_, _, mixed) = convert(&img, &s);
        assert_eq!(mixed[0].color, [180, 100, 40]);
        s.look = Look::Blocks;
        let (_, _, other_look) = convert(&img, &s);
        assert_eq!(other_look[0].color, mixed[0].color);
    }

    #[test]
    fn visor_default_is_visibly_distinct_from_blocks_with_the_same_colour() {
        let img = RgbaImage::from_pixel(32, 32, image::Rgba([100, 100, 100, 255]));
        let mut s = Settings::default();
        s.cols = 16;
        s.color_kind = ColorKind::Source;
        let (_, _, visor) = convert(&img, &s);
        s.look = Look::Blocks;
        let (_, _, blocks) = convert(&img, &s);
        assert!(visor.iter().zip(&blocks).any(|(a, b)| (a.ch == ' ') != (b.ch == ' ')));
    }

    #[test]
    fn visor_texture_changes_midtones_only_for_the_visor_look() {
        let img = RgbaImage::from_pixel(32, 32, image::Rgba([128, 128, 128, 255]));
        let mut s = Settings::default();
        s.cols = 16;
        s.color_kind = ColorKind::Source;
        s.visor_texture = 0.0;
        let (_, _, plain) = convert(&img, &s);
        s.visor_texture = 1.0;
        let (_, _, textured) = convert(&img, &s);
        assert!(textured.iter().zip(&plain).any(|(a, b)| a.ch != b.ch));
        assert!(textured.iter().zip(&plain).all(|(a, b)| a.color == b.color));
        s.look = Look::Blocks;
        let (_, _, blocks_with_texture) = convert(&img, &s);
        s.visor_texture = 0.0;
        let (_, _, blocks_plain) = convert(&img, &s);
        assert_eq!(blocks_with_texture.iter().map(|c| c.ch).collect::<Vec<_>>(), blocks_plain.iter().map(|c| c.ch).collect::<Vec<_>>());
        let white = RgbaImage::from_pixel(32, 32, image::Rgba([255, 255, 255, 255]));
        s.look = Look::Visor;
        s.visor_texture = 1.0;
        let (_, _, white_cells) = convert(&white, &s);
        assert!(white_cells.iter().all(|c| c.ch == '█'));
    }

    #[test]
    fn neutral_adjustments_match_legacy_output_across_looks() {
        let image = RgbaImage::from_fn(32, 24, |x, y| {
            image::Rgba([(x * 8) as u8, (y * 10) as u8, ((x + y) * 4) as u8, 255])
        });
        for look in Look::ALL {
            for mark in [Mark::Cells, Mark::Chars] {
                let mut s = Settings::default();
                look.apply(&mut s);
                s.cols = 16;
                s.mark = mark;
                s.visor_texture = 0.0;
                let (cols, rows, grid) = convert(&image, &s);
                let cw = image.width() as f32 / cols as f32;
                let ch = image.height() as f32 / rows as f32;
                for y in 0..rows {
                    for x in 0..cols {
                        let rgb = enhance(sample(&image, x, y, cw, ch), &s);
                        let yv = luma(rgb);
                        let cell = grid[(y * cols + x) as usize];
                        assert_eq!(cell.ch, glyph(&s, s.charset.ramp(), x, y, yv));
                        assert_eq!(cell.color, cell_color(&s, rgb, yv));
                    }
                }
            }
        }
    }

    #[test]
    fn auto_levels_ignores_isolated_extreme_samples() {
        let mut rgb = vec![[80.0 / 255.0; 3]; 49];
        rgb.extend(vec![[160.0 / 255.0; 3]; 49]);
        rgb.push([0.0; 3]);
        rgb.push([1.0; 3]);
        let samples = Samples { cols: 10, rows: 10, rgb };
        let (low, high) = sample_levels(&samples).unwrap();
        assert!((low - 80.0 / 255.0).abs() < 0.001);
        assert!((high - 160.0 / 255.0).abs() < 0.001);
    }

    #[test]
    fn shaded_cells_add_glyph_density_without_changing_char_output() {
        let image = RgbaImage::from_pixel(20, 20, image::Rgba([100, 100, 100, 255]));
        let mut s = Settings::default();
        s.cols = 8;
        s.visor_texture = 0.0;
        s.color_kind = ColorKind::Source;
        let (_, _, solid) = convert(&image, &s);
        s.shade_cells = true;
        let (_, _, shaded) = convert(&image, &s);
        assert_eq!(solid[0].ch, shaded[0].ch);
        assert!(shaded[0].color[0] < solid[0].color[0]);
        s.mark = Mark::Chars;
        let (_, _, chars_shaded) = convert(&image, &s);
        s.shade_cells = false;
        let (_, _, chars_solid) = convert(&image, &s);
        assert_eq!(chars_shaded.iter().map(|c| (c.ch, c.color)).collect::<Vec<_>>(), chars_solid.iter().map(|c| (c.ch, c.color)).collect::<Vec<_>>());
    }

    #[test]
    fn filter_strength_preserves_defaults_and_restores_highlight_tint() {
        let white = [1.0; 3];
        let mut s = Settings::default();
        s.color_kind = ColorKind::Teal;
        assert_eq!(cell_color(&s, white, 1.0), CREAM_RGB);
        s.filter_strength = 1.0;
        assert_eq!(cell_color(&s, white, 1.0), TEAL_RGB);
        s.color_kind = ColorKind::Coral;
        assert_eq!(cell_color(&s, white, 1.0), CORAL_RGB);
        s.color_kind = ColorKind::Cream;
        let stronger = cell_color(&s, [0.25; 3], 0.25);
        s.filter_strength = 0.0;
        let baseline = cell_color(&s, [0.25; 3], 0.25);
        assert!(stronger[0] > baseline[0]);
        s.color_kind = ColorKind::Source;
        s.filter_strength = 1.0;
        assert_eq!(cell_color(&s, white, 1.0), [255; 3]);
    }

    #[test]
    fn auto_levels_expands_low_contrast_but_leaves_flat_images_alone() {
        let mut img = RgbaImage::from_pixel(20, 20, image::Rgba([80, 80, 80, 255]));
        for y in 0..20 { for x in 10..20 { img.put_pixel(x, y, image::Rgba([160, 160, 160, 255])); } }
        let mut s = Settings::default();
        s.cols = 8;
        s.color_kind = ColorKind::Source;
        let (_, _, before) = convert(&img, &s);
        s.auto_levels = true;
        let (_, _, after) = convert(&img, &s);
        let span = |cells: &[Cell]| cells.iter().map(|c| c.color[0]).max().unwrap() - cells.iter().map(|c| c.color[0]).min().unwrap();
        assert!(span(&after) > span(&before));
        let flat = RgbaImage::from_pixel(20, 20, image::Rgba([100, 100, 100, 255]));
        let (_, _, on) = convert(&flat, &s);
        s.auto_levels = false;
        let (_, _, off) = convert(&flat, &s);
        assert_eq!(on.iter().map(|c| (c.ch, c.color)).collect::<Vec<_>>(), off.iter().map(|c| (c.ch, c.color)).collect::<Vec<_>>());
    }

    #[test]
    fn tone_controls_lift_and_separate_midtone_values() {
        let mut s = Settings::default();
        let original = [0.25, 0.25, 0.25];
        assert_eq!(adjust_tone(original, &s), original);
        s.tone_brightness = 0.1;
        assert!(adjust_tone(original, &s)[0] > original[0]);
        s.tone_brightness = 0.0;
        s.tone_gamma = 2.0;
        assert!(adjust_tone(original, &s)[0] > original[0]);
        s.tone_gamma = 1.0;
        s.tone_contrast = 1.5;
        assert!(adjust_tone(original, &s)[0] < original[0]);
        assert!(adjust_tone([0.75; 3], &s)[0] > 0.75);
    }

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
