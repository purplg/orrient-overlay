use bevy::prelude::*;
use sickle_ui::{prelude::*, ui_builder::UiBuilder};

#[derive(Component, Clone, Default, Debug, UiContext)]
pub struct MarkerSeparator;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<MarkerSeparator>::default());
    }
}

impl DefaultTheme for MarkerSeparator {
    fn default_theme() -> Option<Theme<Self>> {
        Self::theme().into()
    }
}

impl MarkerSeparator {
    fn theme() -> Theme<MarkerSeparator> {
        let base_theme = PseudoTheme::deferred(None, MarkerSeparator::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style: &mut StyleBuilder, theme: &ThemeData) {
        let theme_spacing = theme.spacing;
        let colors = theme.colors();
        style
            .border(UiRect::vertical(Val::Px(theme_spacing.borders.small)))
            .border_color(colors.container(Container::SurfaceMid))
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)))
            .background_color(colors.container(Container::SurfaceLow));
    }

    fn frame() -> impl Bundle {
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                align_content: AlignContent::Center,
                ..default()
            },
            ..default()
        }
    }
}

pub trait UiMarkerSeparatorExt {
    fn marker_separator(&mut self, label: impl Into<String>);
}

impl UiMarkerSeparatorExt for UiBuilder<'_, Entity> {
    fn marker_separator(&mut self, label: impl Into<String>) {
        self.container(MarkerSeparator::frame(), |parent| {
            parent
                .spawn(TextBundle {
                    text: Text::from_section(
                        label.into(),
                        TextStyle {
                            font_size: 16.,
                            ..default()
                        },
                    ),
                    ..default()
                })
                .style()
                .align_content(AlignContent::Center)
                .width(Val::Percent(100.));
        })
        .insert(MarkerSeparator);
    }
}
