use bevy::prelude::*;

use sickle_ui::prelude::*;

#[derive(Component, Clone, Default, Debug, UiContext)]
pub struct MarkerListView;
impl MarkerListView {
    pub(super) fn frame() -> impl Bundle {
        (Name::new("MarkerList View"), NodeBundle::default(), Self)
    }

    fn theme() -> Theme<Self> {
        let base_theme = PseudoTheme::deferred(None, Self::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style: &mut StyleBuilder, _theme: &ThemeData) {
        style.width(Val::Percent(100.)).height(Val::Percent(100.));
    }
}
impl DefaultTheme for MarkerListView {
    fn default_theme() -> Option<Theme<Self>> {
        Self::theme().into()
    }
}

pub(super) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<MarkerListView>::default());
    }
}
