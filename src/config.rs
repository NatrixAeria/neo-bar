use std::borrow::Cow;

#[derive(Debug, Clone, Copy)]
pub enum DockDirection {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub enum ZIndex {
    AboveEverything,
    Normal,
    BelowEverything,
}

#[derive(Debug, Clone)]
pub struct BarBuilder {
    title: Cow<'static, str>,
    docking: DockDirection,
    margin_left: i32,
    margin_right: i32,
    z_index: ZIndex,
    transparency: bool,
    width: u32,
}

impl Default for BarBuilder {
    fn default() -> Self {
        Self {
            title: Cow::Borrowed(env!("CARGO_PKG_NAME")),
            docking: DockDirection::Bottom,
            margin_left: 0,
            margin_right: 0,
            z_index: ZIndex::BelowEverything,
            transparency: false,
            width: 20,
        }
    }
}

macro_rules! g_s_etter {
    ($get_name:ident, $name:ident, $t:ty) => {
        pub fn $name(mut self, $name: $t) -> Self {
            self.$name = $name;
            self
        }
        pub fn $get_name(&self) -> &$t {
            &self.$name
        }
    };
}

impl BarBuilder {
    pub fn title_owned(mut self, title: String) -> Self {
        self.title = Cow::Owned(title);
        self
    }

    pub fn title(mut self, title: &'static str) -> Self {
        self.title = Cow::Borrowed(title);
        self
    }

    pub fn margin(mut self, left: i32, right: i32) -> Self {
        self.margin_left = left;
        self.margin_right = right;
        self
    }

    g_s_etter! {get_docking, docking, DockDirection}
    g_s_etter! {get_margin_left, margin_left, i32}
    g_s_etter! {get_margin_right, margin_right, i32}
    g_s_etter! {get_z_index, z_index, ZIndex}
    g_s_etter! {get_transparency, transparency, bool}
    g_s_etter! {get_width, width, u32}
}
