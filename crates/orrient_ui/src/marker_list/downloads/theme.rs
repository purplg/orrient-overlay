use bevy::color::palettes;
use bevy::prelude::*;
use sickle_ui::prelude::*;

/// The main view for the entire downloads tab area.
#[derive(Component, Clone, Default, Debug, UiContext)]
pub struct DownloadView;
impl DownloadView {
    pub fn frame() -> impl Bundle {
        (Name::new("Downloads View"), NodeBundle::default(), Self)
    }

    fn theme() -> Theme<Self> {
        let base_theme = PseudoTheme::deferred(None, Self::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style: &mut StyleBuilder, _theme: &ThemeData) {
        style
            .flex_direction(FlexDirection::Column)
            .width(Val::Percent(100.))
            .height(Val::Percent(100.));
    }
}
impl DefaultTheme for DownloadView {
    fn default_theme() -> Option<Theme<Self>> {
        Self::theme().into()
    }
}

/// Scrollable list of downloadable marker packs
#[derive(Component, Clone, Default, Debug, UiContext)]
pub(super) struct DownloadList;
impl DownloadList {
    pub(super) fn frame() -> impl Bundle {
        (Name::new("Download List"), NodeBundle::default(), Self)
    }

    fn theme() -> Theme<Self> {
        let base_theme = PseudoTheme::deferred(None, Self::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style: &mut StyleBuilder, _theme: &ThemeData) {
        style
            .flex_direction(FlexDirection::Column)
            .width(Val::Percent(100.))
            .height(Val::Percent(100.));
    }
}
impl DefaultTheme for DownloadList {
    fn default_theme() -> Option<Theme<Self>> {
        Self::theme().into()
    }
}

/// The highest container for a single entry for a downloadable repo
/// pack.
#[derive(Component, Clone, Default, Debug, UiContext)]
pub(super) struct DownloadPack;
impl DownloadPack {
    pub(super) fn frame() -> impl Bundle {
        (Name::new("Marker Entry"), NodeBundle::default(), Self)
    }

    fn theme() -> Theme<Self> {
        let base_theme = PseudoTheme::deferred(None, Self::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style: &mut StyleBuilder, theme: &ThemeData) {
        let theme_spacing = theme.spacing;
        let colors = theme.colors();
        style
            .width(Val::Percent(100.))
            .border(UiRect::all(Val::Px(1.)))
            .border_color(palettes::basic::BLACK)
            .background_color(colors.container(Container::SurfaceMid))
            .margin(UiRect {
                left: Val::Px(theme_spacing.gaps.medium),
                right: Val::Px(theme_spacing.gaps.medium),
                top: Val::Px(0.0),
                bottom: Val::Px(theme_spacing.gaps.medium),
            })
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.large)));
    }
}
impl DefaultTheme for DownloadPack {
    fn default_theme() -> Option<Theme<Self>> {
        Self::theme().into()
    }
}

/// A button bar for the remote repo list.
#[derive(Component, Clone, Default, Debug, UiContext)]
pub(super) struct RepoBar;
impl RepoBar {
    pub(super) fn frame() -> impl Bundle {
        (Self, NodeBundle::default())
    }

    fn theme() -> Theme<Self> {
        let base_theme = PseudoTheme::deferred(None, Self::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style: &mut StyleBuilder, theme: &ThemeData) {
        style
            .padding(UiRect::all(Val::Px(theme.spacing.gaps.small)))
            .width(Val::Percent(100.))
            .flex_direction(FlexDirection::Row);
    }
}
impl DefaultTheme for RepoBar {
    fn default_theme() -> Option<Theme<Self>> {
        Self::theme().into()
    }
}

/// The main area of a single repo entry
#[derive(Component, Clone, Default, Debug, UiContext)]
pub(super) struct Content;
impl Content {
    pub(super) fn frame() -> impl Bundle {
        (Self, NodeBundle::default())
    }

    fn theme() -> Theme<Self> {
        let base_theme = PseudoTheme::deferred(None, Self::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style: &mut StyleBuilder, _theme: &ThemeData) {
        style
            .flex_direction(FlexDirection::Column)
            .width(Val::Percent(100.));
    }
}
impl DefaultTheme for Content {
    fn default_theme() -> Option<Theme<Self>> {
        Self::theme().into()
    }
}

/// A button to on a repo pack.
#[derive(Component, Clone, Default, Debug, UiContext)]
pub(super) struct RepoButton;
impl RepoButton {
    pub(super) fn frame() -> impl Bundle {
        (Self, ButtonBundle::default())
    }

    fn theme() -> Theme<Self> {
        let base_theme = PseudoTheme::deferred(None, Self::primary_style);
        let open_theme = PseudoTheme::deferred_world(vec![PseudoState::Open], Self::open_style);
        let inactive_theme =
            PseudoTheme::deferred_world(vec![PseudoState::Closed], Self::inactive_style);
        let disabled_theme =
            PseudoTheme::deferred_world(vec![PseudoState::Disabled], Self::disabled_style);
        Theme::new(vec![base_theme, open_theme, inactive_theme, disabled_theme])
    }

    fn primary_style(style: &mut StyleBuilder, theme: &ThemeData) {
        style
            .border(UiRect::all(Val::Px(1.)))
            .border_color(Color::BLACK)
            .padding(UiRect::all(Val::Px(theme.spacing.gaps.small)))
            .margin(UiRect::horizontal(Val::Px(theme.spacing.gaps.small)))
            .align_self(AlignSelf::FlexEnd)
            .animated()
            .background_color(AnimatedVals {
                idle: theme.colors().container(Container::SurfaceMid),
                hover: theme.colors().container(Container::SurfaceHigh).into(),
                ..default()
            });
    }

    fn open_style(style: &mut StyleBuilder, _: Entity, _: &Self, world: &World) {
        let theme = world.resource::<ThemeData>().clone();
        let colors = theme.colors();
        style.background_color(colors.container(Container::SurfaceHighest));
    }

    fn inactive_style(style: &mut StyleBuilder, _: Entity, _: &Self, world: &World) {
        let theme = world.resource::<ThemeData>().clone();
        let colors = theme.colors();
        style.animated().background_color(AnimatedVals {
            idle: colors.container(Container::SurfaceLowest),
            hover: colors.container(Container::SurfaceMid).into(),
            ..default()
        });
    }

    fn disabled_style(style: &mut StyleBuilder, _: Entity, _: &Self, world: &World) {
        let theme = world.resource::<ThemeData>().clone();
        let colors = theme.colors();
        style.background_color(colors.surface(Surface::SurfaceDim));
    }
}
impl DefaultTheme for RepoButton {
    fn default_theme() -> Option<Theme<Self>> {
        Self::theme().into()
    }
}

/// The header of a downloadable repo pack.
#[derive(Component, Clone, Default, Debug, UiContext)]
pub(super) struct Header;
impl Header {
    pub(super) fn frame() -> impl Bundle {
        (Self, NodeBundle::default())
    }

    fn theme() -> Theme<Self> {
        let base_theme = PseudoTheme::deferred(None, Self::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(_style: &mut StyleBuilder, _theme: &ThemeData) {}
}
impl DefaultTheme for Header {
    fn default_theme() -> Option<Theme<Self>> {
        Self::theme().into()
    }
}

/// The body of a downloadable repo pack.
#[derive(Component, Clone, Default, Debug, UiContext)]
pub(super) struct Body;
impl Body {
    pub(super) fn frame() -> impl Bundle {
        (Self, NodeBundle::default())
    }

    fn theme() -> Theme<Self> {
        let base_theme = PseudoTheme::deferred(None, Self::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(_style: &mut StyleBuilder, _theme: &ThemeData) {}
}
impl DefaultTheme for Body {
    fn default_theme() -> Option<Theme<Self>> {
        Self::theme().into()
    }
}

/// The footer of a downloadable repo pack.
#[derive(Component, Clone, Default, Debug, UiContext)]
pub(super) struct Footer;
impl Footer {
    pub(super) fn frame() -> impl Bundle {
        (Self, NodeBundle::default())
    }

    fn theme() -> Theme<Self> {
        let base_theme = PseudoTheme::deferred(None, Self::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style: &mut StyleBuilder, theme: &ThemeData) {
        let theme_spacing = theme.spacing;
        style
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)))
            .margin(UiRect::top(Val::Px(theme_spacing.gaps.small)))
            .width(Val::Percent(100.))
            .background_color(Color::BLACK);
    }
}
impl DefaultTheme for Footer {
    fn default_theme() -> Option<Theme<Self>> {
        Self::theme().into()
    }
}

/// The categories of a downloadable repo pack.
#[derive(Component, Clone, Default, Debug, UiContext)]
pub(super) struct Categories;
impl Categories {
    pub(super) fn frame() -> impl Bundle {
        (Self, NodeBundle::default())
    }

    fn theme() -> Theme<Self> {
        let base_theme = PseudoTheme::deferred(None, Self::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style: &mut StyleBuilder, _theme: &ThemeData) {
        style
            .flex_direction(FlexDirection::Column)
            .justify_content(JustifyContent::Center)
            .width(Val::Percent(50.));
    }
}
impl DefaultTheme for Categories {
    fn default_theme() -> Option<Theme<Self>> {
        Self::theme().into()
    }
}

/// The buttons of a downloadable repo pack.
#[derive(Component, Clone, Default, Debug, UiContext)]
pub(super) struct Buttons;
impl Buttons {
    pub(super) fn frame() -> impl Bundle {
        (Self, NodeBundle::default())
    }

    fn theme() -> Theme<Self> {
        let base_theme = PseudoTheme::deferred(None, Self::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style: &mut StyleBuilder, _theme: &ThemeData) {
        style
            .flex_direction(FlexDirection::Row)
            .align_content(AlignContent::Center)
            .justify_content(JustifyContent::FlexEnd)
            .width(Val::Percent(50.));
    }
}
impl DefaultTheme for Buttons {
    fn default_theme() -> Option<Theme<Self>> {
        Self::theme().into()
    }
}

/// The title of a downloadable repo pack.
#[derive(Component, Clone, Default, Debug, UiContext)]
pub(super) struct Title;
impl Title {
    fn theme() -> Theme<Self> {
        let base_theme = PseudoTheme::deferred(None, Self::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style: &mut StyleBuilder, theme: &ThemeData) {
        style
            .font_color(theme.colors().primary)
            .align_self(AlignSelf::FlexStart)
            .width(Val::Percent(50.));
    }
}
impl DefaultTheme for Title {
    fn default_theme() -> Option<Theme<Self>> {
        Self::theme().into()
    }
}

/// The timestamp of a downloadable repo pack.
#[derive(Component, Clone, Default, Debug, UiContext)]
pub(super) struct Timestamp;
impl Timestamp {
    fn theme() -> Theme<Self> {
        let base_theme = PseudoTheme::deferred(None, Self::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style: &mut StyleBuilder, theme: &ThemeData) {
        style
            .font_color(theme.colors().primary)
            .align_self(AlignSelf::FlexEnd)
            .align_content(AlignContent::FlexEnd)
            .width(Val::Percent(50.));
    }
}
impl DefaultTheme for Timestamp {
    fn default_theme() -> Option<Theme<Self>> {
        Self::theme().into()
    }
}

pub(super) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<DownloadView>::default());
        app.add_plugins(ComponentThemePlugin::<DownloadPack>::default());
        app.add_plugins(ComponentThemePlugin::<DownloadList>::default());
        app.add_plugins(ComponentThemePlugin::<Content>::default());
        app.add_plugins(ComponentThemePlugin::<RepoBar>::default());
        app.add_plugins(ComponentThemePlugin::<RepoButton>::default());
        app.add_plugins(ComponentThemePlugin::<Categories>::default());
        app.add_plugins(ComponentThemePlugin::<Buttons>::default());
        app.add_plugins(ComponentThemePlugin::<Header>::default());
        app.add_plugins(ComponentThemePlugin::<Body>::default());
        app.add_plugins(ComponentThemePlugin::<Footer>::default());
        app.add_plugins(ComponentThemePlugin::<Title>::default());
        app.add_plugins(ComponentThemePlugin::<Timestamp>::default());
    }
}
