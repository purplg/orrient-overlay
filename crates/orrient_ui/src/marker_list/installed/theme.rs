use orrient_pathing::prelude::FullMarkerId;

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use sickle_ui::prelude::*;

#[derive(Component, Clone, Default, Debug, UiContext)]
pub struct InstalledView;
impl InstalledView {
    pub fn frame() -> impl Bundle {
        (Name::new("Installed View"), NodeBundle::default(), Self)
    }

    fn theme() -> Theme<Self> {
        let base_theme = PseudoTheme::deferred(None, Self::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style: &mut StyleBuilder, _theme: &ThemeData) {
        style.height(Val::Percent(100.));
    }
}
impl DefaultTheme for InstalledView {
    fn default_theme() -> Option<Theme<Self>> {
        Self::theme().into()
    }
}

#[derive(Component, Clone, Default, Debug, UiContext)]
pub(super) struct MarkerButton {
    pub full_id: FullMarkerId,
    pub has_children: bool,
    pub open: bool,
}
impl MarkerButton {
    pub(super) fn frame() -> impl Bundle {
        ButtonBundle {
            focus_policy: FocusPolicy::Pass,
            ..default()
        }
    }

    fn theme() -> Theme<MarkerButton> {
        let base_theme = PseudoTheme::deferred(None, MarkerButton::primary_style);
        let open_theme =
            PseudoTheme::deferred_world(vec![PseudoState::Open], MarkerButton::open_style);
        let inactive_theme =
            PseudoTheme::deferred_world(vec![PseudoState::Closed], MarkerButton::inactive_style);
        let disabled_theme =
            PseudoTheme::deferred_world(vec![PseudoState::Disabled], MarkerButton::disabled_style);
        Theme::new(vec![base_theme, open_theme, inactive_theme, disabled_theme])
    }

    fn primary_style(style: &mut StyleBuilder, theme: &ThemeData) {
        let theme_spacing = theme.spacing;
        let colors = theme.colors();
        style
            .border(UiRect::all(Val::Px(1.)))
            .border_color(Color::BLACK)
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)))
            .animated()
            .background_color(AnimatedVals {
                idle: colors.container(Container::SurfaceMid),
                hover: colors.container(Container::SurfaceHigh).into(),
                ..default()
            });
    }

    fn open_style(style: &mut StyleBuilder, _: Entity, _: &MarkerButton, world: &World) {
        let theme = world.resource::<ThemeData>().clone();
        let colors = theme.colors();
        style.background_color(colors.container(Container::SurfaceHighest));
    }

    fn inactive_style(style: &mut StyleBuilder, _: Entity, _: &MarkerButton, world: &World) {
        let theme = world.resource::<ThemeData>().clone();
        let colors = theme.colors();
        style.animated().background_color(AnimatedVals {
            idle: colors.container(Container::SurfaceLowest),
            hover: colors.container(Container::SurfaceMid).into(),
            ..default()
        });
    }

    fn disabled_style(style: &mut StyleBuilder, _: Entity, _: &MarkerButton, world: &World) {
        let theme = world.resource::<ThemeData>().clone();
        let colors = theme.colors();
        style.background_color(colors.on(On::Error));
    }
}
impl DefaultTheme for MarkerButton {
    fn default_theme() -> Option<Theme<Self>> {
        Self::theme().into()
    }
}

pub(super) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<InstalledView>::default());
    }
}
