//! The editable project: style, timeline edits and export settings.
//! Stored as `project.json` inside a `.daisycap` package.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::engine::geom::{Rect, Vec2, v2};
use crate::engine::spring::SpringConfig;

pub const PROJECT_VERSION: u32 = 1;

/// sRGB colour with alpha, serialised as `#rrggbbaa`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Color(pub [u8; 4]);

impl Color {
    pub const BLACK: Color = Color([0, 0, 0, 255]);
    pub const WHITE: Color = Color([255, 255, 255, 255]);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color([r, g, b, 255])
    }
    pub fn to_f32(self) -> [f32; 4] {
        self.0.map(|c| c as f32 / 255.0)
    }
    pub fn hex(self) -> String {
        let [r, g, b, a] = self.0;
        if a == 255 { format!("#{r:02x}{g:02x}{b:02x}") } else { format!("#{r:02x}{g:02x}{b:02x}{a:02x}") }
    }
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim_start_matches('#');
        let byte = |i: usize| u8::from_str_radix(s.get(i..i + 2)?, 16).ok();
        match s.len() {
            6 => Some(Color([byte(0)?, byte(2)?, byte(4)?, 255])),
            8 => Some(Color([byte(0)?, byte(2)?, byte(4)?, byte(6)?])),
            _ => None,
        }
    }
}

impl Serialize for Color {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.hex())
    }
}
impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Color::parse(&s).ok_or_else(|| serde::de::Error::custom(format!("bad colour {s:?}")))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Background {
    /// One of the built-in generated wallpapers.
    Wallpaper { id: String },
    Gradient { angle: f64, stops: Vec<(Color, f64)> },
    Color { color: Color },
    Image { path: String },
}

impl Default for Background {
    fn default() -> Self {
        Background::Wallpaper { id: "aurora".into() }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Shadow {
    pub intensity: f64,
    /// Direction the light comes from, degrees (90 = from above).
    pub angle: f64,
    pub distance: f64,
    pub blur: f64,
}

impl Default for Shadow {
    fn default() -> Self {
        Shadow { intensity: 0.75, angle: 90.0, distance: 25.0, blur: 20.0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Inset {
    /// Inner border thickness in recording pixels.
    pub padding: f64,
    pub color: Color,
    pub alpha: f64,
}

impl Default for Inset {
    fn default() -> Self {
        Inset { padding: 0.0, color: Color::BLACK, alpha: 0.5 }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Aspect {
    /// Recording plus padding.
    #[default]
    Auto,
    Ratio { w: u32, h: u32 },
}

impl Aspect {
    pub const PRESETS: [(&'static str, Aspect); 7] = [
        ("Auto", Aspect::Auto),
        ("Wide 16:9", Aspect::Ratio { w: 16, h: 9 }),
        ("Square 1:1", Aspect::Ratio { w: 1, h: 1 }),
        ("Classic 4:3", Aspect::Ratio { w: 4, h: 3 }),
        ("Vertical 9:16", Aspect::Ratio { w: 9, h: 16 }),
        ("Tall 3:4", Aspect::Ratio { w: 3, h: 4 }),
        ("Portrait 4:5", Aspect::Ratio { w: 4, h: 5 }),
    ];
    pub fn ratio(self) -> Option<f64> {
        match self {
            Aspect::Auto => None,
            Aspect::Ratio { w, h } => Some(w as f64 / h as f64),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ClickEffect {
    #[default]
    None,
    /// An expanding soft disc under the cursor.
    Circle,
    /// A shockwave that ripples the screen content.
    Ripple { strength: f64 },
}

/// Flavour of the synthesized click sound.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClickSound {
    /// The original DaisyCap click.
    #[default]
    Classic,
    /// Low and muted.
    Soft,
    /// Bright and snappy.
    Sharp,
    /// A mechanical-switch thud.
    Thock,
    /// A typewriter lever clack.
    Typewriter,
    /// A bubbly pop.
    Pop,
}

impl ClickSound {
    pub const ALL: [ClickSound; 6] = [ClickSound::Classic, ClickSound::Soft, ClickSound::Sharp, ClickSound::Thock, ClickSound::Typewriter, ClickSound::Pop];

    pub fn label(self) -> &'static str {
        match self {
            ClickSound::Classic => "Classic",
            ClickSound::Soft => "Soft",
            ClickSound::Sharp => "Sharp",
            ClickSound::Thock => "Thock",
            ClickSound::Typewriter => "Typewriter",
            ClickSound::Pop => "Pop",
        }
    }
}

/// Flavour of the keyboard sound: a recording of a real board.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeySound {
    /// A muted office keyboard.
    #[default]
    #[serde(alias = "soft", alias = "pop")]
    Office,
    /// A tactile mechanical keyboard.
    #[serde(alias = "mech", alias = "typewriter")]
    Tactile,
}

impl KeySound {
    pub const ALL: [KeySound; 2] = [KeySound::Office, KeySound::Tactile];

    pub fn label(self) -> &'static str {
        match self {
            KeySound::Office => "Office keyboard",
            KeySound::Tactile => "Mech keyboard",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CursorSkin {
    /// The cursor images recorded from the system.
    #[default]
    System,
    /// Crisp vector cursors drawn by the renderer at any size.
    Vector,
    /// A soft touch-style circle.
    Touch,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CursorStyle {
    pub size: f64,
    pub hide: bool,
    pub skin: CursorSkin,
    /// None = follow the raw mouse exactly.
    pub smoothing: Option<SpringConfig>,
    /// How much horizontal motion tilts the cursor.
    pub tilt: f64,
    pub base_rotation: f64,
    /// Hide after this long without movement.
    pub hide_idle_ms: Option<f64>,
    /// DaisyCap back to the starting position in the last N ms (seamless loops).
    pub loop_ms: Option<f64>,
    /// Freeze in the last N ms.
    pub freeze_ms: f64,
    /// DaisyCap onto upcoming click points this early, removing jitter.
    pub click_snap_ms: f64,
    pub click_effect: ClickEffect,
}

impl Default for CursorStyle {
    fn default() -> Self {
        CursorStyle {
            size: 1.5,
            hide: false,
            skin: CursorSkin::System,
            smoothing: Some(SpringConfig::CURSOR_SMOOTH),
            tilt: 0.5,
            base_rotation: 0.0,
            hide_idle_ms: None,
            loop_ms: None,
            freeze_ms: 0.0,
            click_snap_ms: 500.0,
            click_effect: ClickEffect::None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MotionBlur {
    pub amount: f64,
    pub cursor: f64,
    pub screen_move: f64,
    pub screen_zoom: f64,
}

impl Default for MotionBlur {
    fn default() -> Self {
        MotionBlur { amount: 1.0, cursor: 1.0, screen_move: 1.0, screen_zoom: 1.0 }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CameraAspect {
    #[default]
    Original,
    Square,
    Vertical,
    Horizontal,
}

/// The camera bubble's outline.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CameraShape {
    /// Rounded rectangle in the chosen aspect (radius from roundness).
    #[default]
    Rounded,
    /// Circle (centre square crop).
    Circle,
    /// Square, rounded by roundness (centre square crop).
    Square,
    /// Capsule in the chosen aspect, fully round ends.
    Pill,
    /// Superellipse square (continuous corners); roundness sets the curve.
    Squircle,
}

impl CameraShape {
    pub const ALL: [CameraShape; 5] = [CameraShape::Circle, CameraShape::Rounded, CameraShape::Square, CameraShape::Pill, CameraShape::Squircle];

    pub fn label(self) -> &'static str {
        match self {
            CameraShape::Rounded => "Rounded",
            CameraShape::Circle => "Circle",
            CameraShape::Square => "Square",
            CameraShape::Pill => "Pill",
            CameraShape::Squircle => "Squircle",
        }
    }

    /// Crops the camera to a centred square.
    pub fn is_square(self) -> bool {
        matches!(self, CameraShape::Circle | CameraShape::Square | CameraShape::Squircle)
    }

    /// The roundness setting applies.
    pub fn uses_roundness(self) -> bool {
        matches!(self, CameraShape::Rounded | CameraShape::Square | CameraShape::Squircle)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CameraStyle {
    pub hide: bool,
    pub mirror: bool,
    pub shape: CameraShape,
    /// Border width in pixels at 1080p (scaled with the output height).
    pub border: f64,
    pub border_color: Color,
    /// Soft drop shadow under the bubble.
    pub shadow: bool,
    /// Fraction of the safe frame the bubble may occupy.
    pub size: f64,
    /// Corner radius as a fraction of the bubble's short side (0.5 = round).
    pub roundness: f64,
    /// Position inside the safe frame, (0,0) top-left to (1,1) bottom-right.
    pub position: Vec2,
    pub aspect: CameraAspect,
    /// Bubble scale while the screen is zoomed in.
    pub zoom_scale: f64,
}

impl CameraStyle {
    /// Corner radius in pixels for a `w`x`h` bubble.
    pub fn radius(&self, w: f64, h: f64) -> f64 {
        let short = w.min(h);
        match self.shape {
            CameraShape::Circle | CameraShape::Pill => short / 2.0,
            CameraShape::Rounded | CameraShape::Square => self.roundness.clamp(0.0, 0.5) * short,
            CameraShape::Squircle => short / 2.0,
        }
    }

    /// Superellipse exponent (0 = not a squircle): 2 (a circle) at the
    /// roundest, 4 (the classic squircle) halfway, 10 (nearly square) at 0.
    pub fn squircle_exponent(&self) -> f64 {
        if self.shape != CameraShape::Squircle {
            return 0.0;
        }
        let r = (self.roundness / 0.5).clamp(0.0, 1.0);
        2.0 + 8.0 * (1.0 - r).powi(2)
    }

    /// Border width in pixels for an output `out_h` pixels tall.
    pub fn border_px(&self, out_h: f64) -> f64 {
        (self.border.max(0.0) * out_h / 1080.0).round()
    }
}

impl Default for CameraStyle {
    fn default() -> Self {
        CameraStyle {
            hide: false,
            mirror: false,
            shape: CameraShape::Rounded,
            border: 0.0,
            border_color: Color::WHITE,
            shadow: true,
            size: 0.35,
            roundness: 0.25,
            position: v2(1.0, 1.0),
            aspect: CameraAspect::Original,
            zoom_scale: 0.7,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShortcutStyle {
    pub show: bool,
    pub single_keys: bool,
    pub size: f64,
}

impl Default for ShortcutStyle {
    fn default() -> Self {
        ShortcutStyle { show: false, single_keys: false, size: 1.0 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CaptionStyle {
    pub show: bool,
    pub size: f64,
    /// Where the line sits: 0 = bottom, 0.5 = middle, 1 = top.
    pub place: f64,
    /// The dark box behind the words.
    pub backdrop: bool,
    /// Text colour.
    pub color: Color,
    /// Draw the words in capitals.
    pub uppercase: bool,
    /// Pill corners (0 = square) of the backdrop, in output px at 1080p.
    pub radius: f64,
}

impl Default for CaptionStyle {
    fn default() -> Self {
        CaptionStyle { show: true, size: 1.0, place: 0.0, backdrop: true, color: Color::WHITE, uppercase: false, radius: 0.0 }
    }
}

impl CaptionStyle {
    /// Ready-made looks for the caption track.
    pub const PRESETS: [(&'static str, CaptionStyle); 4] = [
        ("Clean", CaptionStyle { size: 1.0, place: 0.0, backdrop: true, color: Color::WHITE, uppercase: false, radius: 0.0, show: true }),
        ("Bold", CaptionStyle { size: 1.35, place: 0.0, backdrop: true, color: Color::WHITE, uppercase: true, radius: 0.0, show: true }),
        ("Minimal", CaptionStyle { size: 0.9, place: 0.0, backdrop: false, color: Color::WHITE, uppercase: false, radius: 0.0, show: true }),
        ("Lower third", CaptionStyle { size: 0.85, place: 0.0, backdrop: true, color: Color::rgb(0xff, 0xe0, 0x8a), uppercase: false, radius: 6.0, show: true }),
    ];
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioStyle {
    pub volume: f64,
    pub mute_mic: bool,
    pub mute_system: bool,
    /// Gentle denoise + level for the microphone track.
    pub enhance_mic: bool,
    pub music: Option<String>,
    pub music_volume: f64,
    pub click_sounds: bool,
    pub click_sound: ClickSound,
    pub click_volume: f64,
    /// Keyboard sounds at every key press.
    pub key_sounds: bool,
    pub key_sound: KeySound,
    pub key_volume: f64,
    /// Only keys typed without ⌘/⌃: shortcut bursts stay quiet.
    pub key_typing_only: bool,
    /// A soft whoosh when a zoom starts.
    pub zoom_whoosh: bool,
    pub whoosh_volume: f64,
    /// Lower the recorded audio while a voice-over plays.
    pub duck_under_voiceover: bool,
    /// Level of the recorded audio under a voice-over (0..1).
    pub duck_level: f64,
}

impl Default for AudioStyle {
    fn default() -> Self {
        AudioStyle {
            volume: 1.0,
            mute_mic: false,
            mute_system: false,
            enhance_mic: true,
            music: None,
            music_volume: 0.05,
            click_sounds: false,
            click_sound: ClickSound::default(),
            click_volume: 0.25,
            key_sounds: false,
            key_sound: KeySound::default(),
            key_volume: 0.2,
            key_typing_only: false,
            zoom_whoosh: false,
            whoosh_volume: 0.35,
            duck_under_voiceover: true,
            duck_level: 0.15,
        }
    }
}

/// A generated card at the start or end of the video: the project's own
/// background, a title, and the daisy mark. Nothing to film or design.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Card {
    pub title: String,
    /// A smaller second line under the title.
    pub subtitle: String,
    /// How long it holds, ms.
    pub ms: f64,
}

impl Default for Card {
    fn default() -> Self {
        Card { title: String::new(), subtitle: String::new(), ms: 2500.0 }
    }
}

impl Card {
    /// The longest a card may hold, so a stray slider can't make a minute of it.
    pub const MAX_MS: f64 = 10_000.0;
    /// How long the fade at each end lasts, ms.
    pub const FADE_MS: f64 = 260.0;

    /// Text opacity at `t_ms` into a card `ms` long.
    pub fn alpha(t_ms: f64, ms: f64) -> f64 {
        let fade = Self::FADE_MS.min(ms / 3.0);
        let in_ = (t_ms / fade).clamp(0.0, 1.0);
        let out = ((ms - t_ms) / fade).clamp(0.0, 1.0);
        (in_ * out).sqrt()
    }
}

/// A device mockup drawn around the screen (generated, not artwork).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeviceFrame {
    #[default]
    None,
    Laptop,
    Phone,
    Tablet,
}

/// The visitor's own branding: a logo and two colours, used on the end
/// credit and in place of the daisy mark.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Brand {
    /// A logo image inside the package (`assets/…`), or an absolute path.
    pub logo: Option<String>,
    /// Accent used on the credit.
    pub color: Color,
    /// The credit card's backdrop.
    pub background: Color,
}

impl Default for Brand {
    fn default() -> Self {
        Brand { logo: None, color: Color::rgb(0xff, 0xc8, 0x6b), background: Color::rgb(0x0b, 0x0b, 0x0f) }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Style {
    pub background: Background,
    pub background_blur: f64,
    /// Space around the window, percent of the mean screen dimension.
    pub padding: f64,
    /// Window corner radius in recording pixels.
    pub corner_radius: f64,
    pub shadow: Shadow,
    pub inset: Inset,
    pub aspect: Aspect,
    /// Normalised crop of the recording.
    pub crop: Rect,
    pub cursor: CursorStyle,
    pub screen_spring: SpringConfig,
    pub motion_blur: MotionBlur,
    /// Dip-to-background dissolve at each cut, ms (0 = hard cuts).
    pub transition_ms: f64,
    /// With a forced frame shape, pan the video so the action stays in frame.
    pub auto_reframe: bool,
    pub camera: CameraStyle,
    pub shortcuts: ShortcutStyle,
    pub captions: CaptionStyle,
    pub audio: AudioStyle,
    /// The visitor's own logo and colours.
    pub brand: Brand,
    /// Zoom level used for new automatic zooms.
    pub default_zoom: f64,
    pub device: DeviceFrame,
    /// The DaisyCap daisy mark in the bottom-left corner of the video.
    pub watermark: bool,
    /// The app's own credit at the very end: a dark card with the daisy.
    pub credit: bool,
    /// How long the credit holds, ms.
    pub credit_ms: f64,
    /// A title card played before the recording, and an end card after it.
    pub intro: Option<Card>,
    pub outro: Option<Card>,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            background: Background::default(),
            background_blur: 0.0,
            padding: 10.0,
            corner_radius: 12.0,
            shadow: Shadow::default(),
            inset: Inset::default(),
            aspect: Aspect::Auto,
            crop: Rect::UNIT,
            cursor: CursorStyle::default(),
            screen_spring: SpringConfig::SCREEN_FOCUSED,
            motion_blur: MotionBlur::default(),
            transition_ms: 180.0,
            auto_reframe: true,
            camera: CameraStyle::default(),
            shortcuts: ShortcutStyle::default(),
            captions: CaptionStyle::default(),
            audio: AudioStyle::default(),
            brand: Brand::default(),
            default_zoom: 2.0,
            device: DeviceFrame::None,
            watermark: true,
            credit: true,
            credit_ms: 3400.0,
            intro: None,
            outro: None,
        }
    }
}

impl Style {
    /// Padding used to bring a hidden background into view.
    pub const REVEAL_PADDING: f64 = 8.0;

    /// With 0 % padding the background (and shadow) can't be seen. If an
    /// edit since `before` changed the background, its blur or the shadow,
    /// add padding (and soft corners) so the change shows. Returns whether
    /// it did.
    pub fn reveal_background(&mut self, before: &Style) -> bool {
        let touched = self.background != before.background || self.background_blur != before.background_blur || self.shadow != before.shadow;
        if !touched || self.padding > 0.0 || self.device != DeviceFrame::None {
            return false;
        }
        self.padding = Style::REVEAL_PADDING;
        if self.corner_radius == 0.0 {
            self.corner_radius = 12.0;
        }
        true
    }
}

/// A kept piece of the recording. Gaps between slices are cuts.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Slice {
    pub id: u64,
    /// Source (recording) time range, ms.
    pub start: f64,
    pub end: f64,
    pub speed: f64,
    /// Ease the speed in and out instead of a hard jump to `speed`.
    pub ease: bool,
    pub mic_volume: f64,
    pub system_volume: f64,
    /// Audio fade at this slice's playback edges, ms (keeps cuts click-free).
    pub fade_in_ms: f64,
    pub fade_out_ms: f64,
    pub hide_cursor: bool,
    /// Follow the raw mouse in this slice.
    pub no_smoothing: bool,
}

impl Default for Slice {
    fn default() -> Self {
        Slice {
            id: 0,
            start: 0.0,
            end: 0.0,
            speed: 1.0,
            ease: false,
            mic_volume: 1.0,
            system_volume: 1.0,
            fade_in_ms: 0.0,
            fade_out_ms: 0.0,
            hide_cursor: false,
            no_smoothing: false,
        }
    }
}

impl Slice {
    pub fn source_len(&self) -> f64 {
        self.end - self.start
    }
    pub fn playback_len(&self) -> f64 {
        self.source_len() / self.speed
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ZoomMode {
    /// Follow clusters of clicks and activity.
    #[default]
    Auto,
    /// Zoom on a fixed point (normalised in the cropped screen).
    Manual { target: Vec2 },
    /// Keep the live cursor in view.
    FollowMouse,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Zoom {
    pub id: u64,
    /// Source time range, ms.
    pub start: f64,
    pub end: f64,
    pub scale: f64,
    pub mode: ZoomMode,
    /// How strongly targets near an edge push the view to that edge.
    pub snap_edges: f64,
    /// Cut in/out instead of animating.
    pub instant: bool,
    pub disabled: bool,
}

impl Default for Zoom {
    fn default() -> Self {
        Zoom {
            id: 0,
            start: 0.0,
            end: 0.0,
            scale: 2.0,
            mode: ZoomMode::Auto,
            snap_edges: 0.25,
            instant: false,
            disabled: false,
        }
    }
}

impl Zoom {
    pub const MIN_LEN_MS: f64 = 1000.0;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LayoutKind {
    /// Screen with the camera bubble.
    #[default]
    Both,
    ScreenOnly,
    CameraOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutRange {
    pub id: u64,
    pub start: f64,
    pub end: f64,
    pub kind: LayoutKind,
    pub camera_size: Option<f64>,
    pub camera_position: Option<Vec2>,
}

impl Default for LayoutRange {
    fn default() -> Self {
        LayoutRange { id: 0, start: 0.0, end: 0.0, kind: LayoutKind::Both, camera_size: None, camera_position: None }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MaskKind {
    /// Soft blur inside the rect (hide private details).
    #[default]
    Blur,
    /// Mosaic inside the rect.
    Pixelate,
    /// Dim everything outside the rect (spotlight).
    Highlight,
    /// Blur painted freehand: `Mask::dabs` rather than the rect.
    Brush,
}

/// A region effect over a time range (source ms).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Mask {
    pub id: u64,
    pub start: f64,
    pub end: f64,
    pub kind: MaskKind,
    /// Region normalised to the recording (0..1, top-left origin).
    pub rect: Rect,
    /// Blur/pixel size in output px at 1080p (Blur, Pixelate) or dim
    /// opacity 0..1 (Highlight).
    pub amount: f64,
    /// Corner radius in output px at 1080p.
    pub radius: f64,
    /// Freehand strokes for `MaskKind::Brush`: (x, y, radius) normalised to
    /// the recording, in the order they were painted.
    pub dabs: Vec<[f32; 3]>,
    /// Radius of a new dab, normalised to the recording's width.
    pub brush: f64,
}

impl Default for Mask {
    fn default() -> Self {
        Mask { id: 0, start: 0.0, end: 0.0, kind: MaskKind::Blur, rect: Rect::new(0.35, 0.35, 0.3, 0.3), amount: 16.0, radius: 10.0, dabs: vec![], brush: 0.05 }
    }
}

impl Mask {
    /// Fade in/out time at the ends of the range.
    pub const FADE_MS: f64 = 150.0;
    pub const MIN_LEN_MS: f64 = 300.0;

    pub fn default_amount(kind: MaskKind) -> f64 {
        match kind {
            MaskKind::Blur => 16.0,
            MaskKind::Pixelate => 14.0,
            MaskKind::Highlight => 0.6,
            MaskKind::Brush => 16.0,
        }
    }

    /// Opacity at a source time (0 outside the range, eased at the ends).
    /// No fade where the range touches the start or end (`duration`) of the
    /// recording, so a mask covering the first frame is there from frame one.
    pub fn alpha_at(&self, t: f64, duration: f64) -> f64 {
        if t < self.start || t > self.end {
            return 0.0;
        }
        let from_start = if self.start <= 1.0 { f64::INFINITY } else { t - self.start };
        let to_end = if self.end >= duration - 1.0 { f64::INFINITY } else { self.end - t };
        let edge = (from_start.min(to_end) / Self::FADE_MS).clamp(0.0, 1.0);
        edge * edge * (3.0 - 2.0 * edge)
    }
}

/// A pointer arrow, an outlined box or a numbered pin drawn over the screen
/// for a time range, following the window and its zoom.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CalloutKind {
    /// An arrow from one point to another (rect start → rect end).
    #[default]
    Arrow,
    /// An outlined rectangle.
    Box,
    /// A filled numbered pin (a step in a walkthrough).
    Step,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Callout {
    pub id: u64,
    pub start: f64,
    pub end: f64,
    pub kind: CalloutKind,
    /// Normalised to the recording. For an arrow, the tip is the far corner.
    pub rect: Rect,
    /// The number on a step pin, starting at 1. Arrows and boxes ignore it.
    pub step: u32,
    /// An optional label drawn beside the shape.
    pub text: String,
    pub color: Color,
    /// Line width in output px at 1080p.
    pub width: f64,
}

impl Default for Callout {
    fn default() -> Self {
        Callout {
            id: 0,
            start: 0.0,
            end: 0.0,
            kind: CalloutKind::Arrow,
            rect: Rect::new(0.28, 0.32, 0.26, 0.2),
            step: 1,
            text: String::new(),
            color: Color::rgb(0xff, 0x45, 0x5e),
            width: 7.0,
        }
    }
}

impl Callout {
    pub const FADE_MS: f64 = 150.0;
    pub const MIN_LEN_MS: f64 = 300.0;

    /// Opacity over the range, fading at both ends.
    pub fn alpha_at(&self, t: f64, duration: f64) -> f64 {
        if t < self.start || t > self.end {
            return 0.0;
        }
        let from_start = if self.start <= 1.0 { f64::INFINITY } else { t - self.start };
        let to_end = if self.end >= duration - 1.0 { f64::INFINITY } else { self.end - t };
        let edge = (from_start.min(to_end) / Self::FADE_MS).clamp(0.0, 1.0);
        edge * edge * (3.0 - 2.0 * edge)
    }

    /// The label shown on the timeline.
    pub fn label(&self) -> &'static str {
        match self.kind {
            CalloutKind::Arrow => "Arrow",
            CalloutKind::Box => "Box",
            CalloutKind::Step => "Step",
        }
    }
}

/// Narration recorded in the editor, played at normal speed from a point on
/// the timeline.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VoiceOver {
    pub id: u64,
    /// Audio file inside the package's recording/ folder.
    pub file: String,
    /// Where it starts (source ms).
    pub start: f64,
    pub duration: f64,
    pub volume: f64,
}

impl Default for VoiceOver {
    fn default() -> Self {
        VoiceOver { id: 0, file: String::new(), start: 0.0, duration: 0.0, volume: 1.0 }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Timeline {
    pub slices: Vec<Slice>,
    pub zooms: Vec<Zoom>,
    pub layouts: Vec<LayoutRange>,
    pub masks: Vec<Mask>,
    pub callouts: Vec<Callout>,
    pub voiceovers: Vec<VoiceOver>,
    next_id: u64,
}

impl Timeline {
    pub fn new_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExportFormat {
    #[default]
    Mp4,
    Gif,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Quality {
    Studio,
    #[default]
    Social,
    Web,
    WebLow,
}

impl Quality {
    pub const ALL: [Quality; 4] = [Quality::Studio, Quality::Social, Quality::Web, Quality::WebLow];

    /// VideoToolbox constant-quality setting (0..1): the encoder spends what
    /// each frame needs, so fine text stays crisp through motion while static
    /// screens stay small.
    pub fn encoder_quality(self) -> f64 {
        match self {
            Quality::Studio => 0.95,
            Quality::Social => 0.85,
            Quality::Web => 0.7,
            Quality::WebLow => 0.55,
        }
    }

    /// Bits per pixel per frame, for size estimates (and GIF effort).
    pub fn bits_per_pixel(self) -> f64 {
        match self {
            Quality::Studio => 0.3,
            Quality::Social => 0.05,
            Quality::Web => 0.0175,
            Quality::WebLow => 0.0075,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Quality::Studio => "Studio",
            Quality::Social => "Social media",
            Quality::Web => "Web",
            Quality::WebLow => "Web (low)",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Codec {
    #[default]
    H264,
    Hevc,
    /// Apple ProRes 4444 in a .mov: visually lossless, full 4:4:4 colour,
    /// for masters and further editing (large files).
    Prores,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExportSettings {
    pub format: ExportFormat,
    /// Output height in pixels; width follows the project's aspect ratio.
    /// 0 = native: the recording's own pixels, unscaled.
    pub height: u32,
    pub fps: u32,
    pub quality: Quality,
    pub codec: Codec,
    pub gif_loop: bool,
    /// MP4 only: pick the bitrate so the file comes out at about this many MB.
    pub target_mb: Option<u32>,
}

impl Default for ExportSettings {
    fn default() -> Self {
        ExportSettings {
            format: ExportFormat::Mp4,
            height: Self::NATIVE,
            // Light mode (older Macs): half the frames to render and encode.
            fps: if crate::hw::light() { 30 } else { 60 },
            quality: Quality::Social,
            codec: Codec::H264,
            gif_loop: true,
            target_mb: None,
        }
    }
}

impl ExportSettings {
    pub const MP4_FPS: [u32; 7] = [60, 50, 30, 25, 24, 20, 10];
    pub const GIF_FPS: [u32; 6] = [50, 30, 25, 20, 15, 10];
    /// `height` for the recording's native resolution.
    pub const NATIVE: u32 = 0;
    pub const MP4_HEIGHTS: [u32; 5] = [Self::NATIVE, 2160, 1440, 1080, 720];
    pub const GIF_HEIGHTS: [u32; 3] = [480, 720, 1080];

}

/// A one-click export recipe for a place you are posting to. It carries the
/// frame shape as well, since social wants square or vertical.
#[derive(Clone, Copy)]
pub struct ExportPreset {
    pub name: &'static str,
    pub hint: &'static str,
    pub aspect: Aspect,
    pub settings: ExportSettings,
}

impl ExportPreset {
    pub const ALL: [ExportPreset; 5] = [
        ExportPreset {
            name: "X / Twitter",
            hint: "1080p MP4, 16:9, 60 fps",
            aspect: Aspect::Ratio { w: 16, h: 9 },
            settings: ExportSettings { fps: 60, height: 1080, quality: Quality::Social, ..ExportSettings::hd() },
        },
        ExportPreset {
            name: "LinkedIn",
            hint: "Square 1:1 MP4 for the feed, 30 fps",
            aspect: Aspect::Ratio { w: 1, h: 1 },
            settings: ExportSettings { fps: 30, height: 1080, quality: Quality::Social, ..ExportSettings::hd() },
        },
        ExportPreset {
            name: "Shorts / Stories",
            hint: "Vertical 9:16 MP4, 30 fps",
            aspect: Aspect::Ratio { w: 9, h: 16 },
            settings: ExportSettings { fps: 30, height: 1080, quality: Quality::Social, ..ExportSettings::hd() },
        },
        ExportPreset {
            name: "YouTube 4K",
            hint: "2160p MP4, 16:9, 60 fps, high quality",
            aspect: Aspect::Ratio { w: 16, h: 9 },
            settings: ExportSettings { fps: 60, height: 2160, quality: Quality::Studio, ..ExportSettings::hd() },
        },
        ExportPreset {
            name: "GIF for Slack",
            hint: "Small looping GIF, 480p",
            aspect: Aspect::Ratio { w: 16, h: 9 },
            settings: ExportSettings { format: ExportFormat::Gif, fps: 15, height: 480, quality: Quality::Studio, ..ExportSettings::hd() },
        },
    ];
}

impl ExportSettings {
    /// The base for a preset: MP4, H.264, at the given size (const-friendly,
    /// so the presets can live in a `const`).
    pub const fn hd() -> ExportSettings {
        ExportSettings {
            format: ExportFormat::Mp4,
            height: Self::NATIVE,
            fps: 60,
            quality: Quality::Social,
            codec: Codec::H264,
            gif_loop: true,
            target_mb: None,
        }
    }
}

impl ExportSettings {
    pub fn gif_default() -> Self {
        ExportSettings { format: ExportFormat::Gif, height: 480, fps: 15, quality: Quality::Studio, ..Default::default() }
    }

    /// Whether "fit to file size" applies (not to GIF or ProRes).
    pub fn fits_size(&self) -> bool {
        self.format == ExportFormat::Mp4 && self.codec != Codec::Prores
    }

    /// The codec actually used at a size: VideoToolbox's H.264 encoder stops
    /// at 4096x2304, so bigger (e.g. native Retina) exports use HEVC.
    pub fn codec_for(&self, width: u32, height: u32) -> Codec {
        let (long, short) = (width.max(height), width.min(height));
        if self.codec == Codec::H264 && (long > 4096 || short > 2304) { Codec::Hevc } else { self.codec }
    }

    /// Audio bitrate assumed for size estimates (AAC stereo).
    pub const AUDIO_BPS: f64 = 192_000.0;

    /// Target video bitrate (bits/s) for an output size and duration. With a
    /// target size the bitrate fills that size (minus audio and container
    /// overhead); otherwise it follows the quality preset.
    pub fn bitrate(&self, width: u32, height: u32, secs: f64) -> u64 {
        if let Some(mb) = self.target_mb.filter(|_| self.fits_size())
            && secs > 0.0
        {
            let total = mb as f64 * 8e6 * 0.96 / secs;
            return (total - Self::AUDIO_BPS).max(250_000.0) as u64;
        }
        // ProRes 4444 is fixed-quality intra-frame (~2-3 bits/pixel on screens).
        let bpp = if self.codec == Codec::Prores { 2.5 } else { self.quality.bits_per_pixel() };
        let base = width as f64 * height as f64 * self.fps as f64 * bpp;
        // HEVC reaches the same quality with ~40% fewer bits.
        let codec = if self.codec_for(width, height) == Codec::Hevc { 0.6 } else { 1.0 };
        (base * codec).max(250_000.0) as u64
    }

    /// Rough output size in MB.
    pub fn estimate_mb(&self, width: u32, height: u32, secs: f64) -> f64 {
        match self.format {
            ExportFormat::Mp4 => (self.bitrate(width, height, secs) as f64 + Self::AUDIO_BPS) * secs / 8e6,
            // Delta-frame GIFs of screen content average ~0.035 bytes per pixel per frame.
            ExportFormat::Gif => (width * height) as f64 * secs * self.fps as f64 * 0.035 / 1e6,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Project {
    pub version: u32,
    pub id: String,
    pub name: String,
    pub created_ms: i64,
    pub updated_ms: i64,
    pub style: Style,
    pub timeline: Timeline,
    pub export: ExportSettings,
}

impl Default for Project {
    fn default() -> Self {
        Project {
            version: PROJECT_VERSION,
            id: String::new(),
            name: "Untitled".into(),
            created_ms: 0,
            updated_ms: 0,
            style: Style::default(),
            timeline: Timeline::default(),
            export: ExportSettings::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brand_round_trips() {
        let mut s = Style::default();
        s.brand.logo = Some("assets/logo-1.png".into());
        s.brand.color = Color::rgb(0x7d, 0xb8, 0xff);
        let json = serde_json::to_string(&s).unwrap();
        let back: Style = serde_json::from_str(&json).unwrap();
        assert_eq!(back.brand, s.brand);
        assert!(Style::default().brand.logo.is_none(), "the daisy is the default mark");
    }

    #[test]
    fn card_round_trips_and_old_projects_still_load() {
        let mut s = Style::default();
        s.intro = Some(Card { title: "Hi".into(), subtitle: "there".into(), ms: 3000.0 });
        let json = serde_json::to_string(&s).unwrap();
        let back: Style = serde_json::from_str(&json).unwrap();
        assert_eq!(back.intro, s.intro);
        // A style written before the card fields existed still loads.
        let style: Style = serde_json::from_str("{}").unwrap();
        assert!(style.intro.is_none() && style.outro.is_none());
        // A card missing its own fields falls back to the defaults.
        let card: Card = serde_json::from_str("{\"title\":\"Only a title\"}").unwrap();
        assert_eq!(card.ms, Card::default().ms);
    }

    #[test]
    fn card_alpha_fades_in_and_out() {
        assert_eq!(Card::alpha(0.0, 2000.0), 0.0);
        assert_eq!(Card::alpha(2000.0, 2000.0), 0.0);
        assert!((Card::alpha(1000.0, 2000.0) - 1.0).abs() < 1e-9);
        // Never past full opacity, never negative.
        for t in [0.0, 50.0, 130.0, 900.0, 1870.0, 1999.0, 2000.0] {
            let a = Card::alpha(t, 2000.0);
            assert!((0.0..=1.0).contains(&a), "alpha {a} at {t}");
        }
    }

    #[test]
    fn callout_fades_and_round_trips() {
        let c = Callout { start: 500.0, end: 2500.0, ..Default::default() };
        assert_eq!(c.alpha_at(400.0, 10_000.0), 0.0);
        assert_eq!(c.alpha_at(2600.0, 10_000.0), 0.0);
        assert!((c.alpha_at(1500.0, 10_000.0) - 1.0).abs() < 1e-9);
        let mut t = Timeline::default();
        t.callouts.push(c.clone());
        let json = serde_json::to_string(&t).unwrap();
        let back: Timeline = serde_json::from_str(&json).unwrap();
        assert_eq!(back.callouts, t.callouts);
    }

    #[test]
    fn export_presets_stay_valid() {
        for p in ExportPreset::ALL {
            let (heights, fpss): (&[u32], &[u32]) = match p.settings.format {
                ExportFormat::Mp4 => (&ExportSettings::MP4_HEIGHTS, &ExportSettings::MP4_FPS),
                ExportFormat::Gif => (&ExportSettings::GIF_HEIGHTS, &ExportSettings::GIF_FPS),
            };
            assert!(heights.contains(&p.settings.height), "{}: height {}", p.name, p.settings.height);
            assert!(fpss.contains(&p.settings.fps), "{}: fps {}", p.name, p.settings.fps);
            // A preset that leaves the shape alone would do nothing you can see.
            assert!(p.aspect != Aspect::Auto, "{} has no shape", p.name);
            assert!(!p.name.is_empty() && !p.hint.is_empty(), "{} needs a hint", p.name);
        }
    }

    #[test]
    fn colour_round_trip() {
        let c = Color([1, 0xab, 0xff, 0x80]);
        assert_eq!(Color::parse(&c.hex()), Some(c));
        assert_eq!(Color::parse("#ffffff"), Some(Color::WHITE));
    }

    #[test]
    fn project_round_trips_through_json() {
        let mut p = Project::default();
        p.style.background = Background::Gradient { angle: 135.0, stops: vec![(Color::BLACK, 0.0), (Color::WHITE, 1.0)] };
        p.timeline.zooms.push(Zoom { mode: ZoomMode::Manual { target: v2(0.2, 0.3) }, ..Default::default() });
        let s = serde_json::to_string(&p).unwrap();
        let q: Project = serde_json::from_str(&s).unwrap();
        assert_eq!(p, q);
    }

    #[test]
    fn missing_fields_take_defaults() {
        let p: Project = serde_json::from_str(r#"{"name":"x","style":{"padding":3}}"#).unwrap();
        assert_eq!(p.style.padding, 3.0);
        assert_eq!(p.style.corner_radius, 12.0);
        assert_eq!(p.export.fps, ExportSettings::default().fps);
        assert_eq!(p.export.height, ExportSettings::NATIVE);
    }

    #[test]
    fn bitrate_formula() {
        let e = ExportSettings { height: 1080, fps: 60, ..Default::default() };
        assert_eq!(e.bitrate(1920, 1080, 10.0), (1920.0 * 1080.0 * 60.0 * 0.05) as u64);
        // Fit to 10 MB over 20 s: ~3.84 Mbit/s total minus audio.
        let t = ExportSettings { target_mb: Some(10), ..e };
        assert_eq!(t.bitrate(1920, 1080, 20.0), (10.0 * 8e6 * 0.96 / 20.0 - ExportSettings::AUDIO_BPS) as u64);
        assert!((t.estimate_mb(1920, 1080, 20.0) - 9.6).abs() < 0.01);
    }

    #[test]
    fn big_exports_switch_to_hevc() {
        let e = ExportSettings::default();
        assert_eq!(e.codec_for(3840, 2160), Codec::H264);
        assert_eq!(e.codec_for(4654, 3200), Codec::Hevc);
        assert_eq!(e.codec_for(2160, 3840), Codec::H264);
        assert_eq!(ExportSettings { codec: Codec::Hevc, ..e }.codec_for(1280, 720), Codec::Hevc);
    }
}
