#![allow(unused)]

use std::convert::identity;

use byteorder::{LittleEndian, ReadBytesExt};
use log::warn;
use quick_xml::{events::attributes::Attributes, name::QName};
use serde::{Deserialize, Deserializer};

use super::Error;

#[derive(Clone, Deserialize, Debug)]
pub(super) struct OverlayData {
    #[serde(rename = "MarkerCategory", default)]
    pub categories: Vec<MarkerCategory>,
    #[serde(rename = "POIs", default)]
    pub pois: Vec<POIs>,
}

#[derive(Clone, Deserialize, Debug, Default)]
// #[serde(deny_unknown_fields)]
pub(super) struct MarkerCategory {
    #[serde(rename = "@name", alias = "@Name")]
    _name: Option<String>,
    #[serde(rename = "@DisplayName")]
    _display_name: Option<String>,
    #[serde(rename = "@IsSeparator", default)]
    pub is_separator: bool,
    #[serde(rename = "MarkerCategory", default)]
    pub categories: Vec<MarkerCategory>,
    #[serde(rename = "@fadeNear", default)]
    pub fade_near: Option<f32>,
    #[serde(rename = "@fadeFar", default)]
    pub fade_far: Option<f32>,
    #[serde(rename = "@iconFile", default)]
    pub icon_path: Option<String>,
    #[serde(rename = "@iconSize", default)]
    pub icon_size: Option<f32>,
    #[serde(rename = "@mapDisplaySize", default)]
    pub map_display_size: Option<f32>,
    #[serde(rename = "@inGameVisibility", default)]
    pub show_on_ingame: bool,
    #[serde(rename = "@mapVisibility", default)]
    pub show_on_map: bool,
    #[serde(rename = "@miniMapVisibility", default)]
    pub show_on_minimap: bool,
    #[serde(rename = "@heightOffset", default)]
    pub height_offset: Option<f32>,
    #[serde(rename = "@minSize", default)]
    pub min_size: Option<f32>,
    #[serde(
        rename = "@achievementId",
        deserialize_with = "achievement_deser",
        default
    )]
    pub achievement_id: Option<u32>,
    #[serde(rename = "@achievementBit", default)]
    pub achievement_bit: Option<u8>,
    #[serde(rename = "@bounce", default)]
    pub bounce: Option<String>,
    #[serde(rename = "@bounce-height", default)]
    pub bounce_height: Option<f32>,
    #[serde(rename = "@autotrigger", default)]
    pub autotrigger: bool,
    #[serde(rename = "@triggerrange", default)]
    pub triggerrange: Option<f32>,
    #[serde(rename = "@tip-name", default)]
    pub tip_name: Option<String>,
    #[serde(rename = "@tip-description", default)]
    pub tip_description: Option<String>,
    #[serde(rename = "@behavior", default)]
    pub behavior: Option<u8>,
    #[serde(rename = "@copy", default)]
    pub copy: Option<String>,
    #[serde(rename = "@copy-message", default)]
    pub copy_message: Option<String>,
    #[serde(rename = "@resetLength", default)]
    pub reset_length: Option<f32>,
    #[serde(rename = "@toggleCategory", default)]
    pub toggle_category: Option<String>,
    #[serde(rename = "@profession", default)]
    pub profession: Option<String>,

    #[serde(rename = "@bh-name")]
    bh_name: Option<String>,
    #[serde(rename = "@bh-DisplayName")]
    bh_display_name: Option<String>,
    #[serde(rename = "@bh-heightOffset", default)]
    pub bh_height_offset: Option<f32>,
    #[serde(rename = "@bh-iconSize", default)]
    pub bh_icon_size: Option<f32>,
    #[serde(rename = "@bh-inGameVisibility", default)]
    pub bh_show_on_ingame: bool,
    #[serde(rename = "@bh-mapVisibility", default)]
    pub bh_show_on_map: bool,
    #[serde(rename = "@bh-miniMapVisibility", default)]
    pub bh_show_on_minimap: bool,
}

impl MarkerCategory {
    pub(super) fn from_attrs(attrs: Attributes) -> Self {
        let mut category = Self::default();

        for attr in attrs.map(Result::ok).filter_map(identity) {
            println!("attr: {:?}", attr);
            let Ok(key) = String::from_utf8(attr.key.0.to_vec()) else {
                warn!("Key is not UTF-8 encoded: {:?}", attr);
                continue;
            };

            let Ok(value) = String::from_utf8(attr.value.to_vec()) else {
                warn!("Value is not UTF-8 encoded: {:?}", attr);
                continue;
            };

            match key.to_lowercase().as_str() {
                "name" => {
                    category._name = String::from_utf8(attr.value.to_vec()).ok();
                }
                "displayname" => {
                    category._display_name = String::from_utf8(attr.value.to_vec()).ok();
                }
                "isseparator" => {
                    category.is_separator = if value.to_lowercase() == "true" {
                        true
                    } else {
                        false
                    }
                }
                _ => println!("Unknown attribute: {:?}", attr),
            }
        }
        category
    }
}

/// There is a single instance where an achievementId is "XXX" so I
/// just set to a default value if it fails.
/// Located in the `tw_mc_masterypoints.xml` markers.
fn achievement_deser<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(u32::deserialize(deserializer).ok())
}

impl MarkerCategory {
    pub(super) fn id(&self) -> String {
        self._name
            .clone()
            .or_else(|| self.bh_name.clone())
            .expect("No category name")
            .clone()
    }

    pub(super) fn display_name(&self) -> String {
        self._display_name
            .clone()
            .or_else(|| self.bh_display_name.clone())
            .expect("No category name")
            .clone()
    }
}

#[derive(Deserialize, Clone, Debug)]
pub(super) struct POIs {
    #[serde(rename = "POI", default)]
    pub poi: Vec<Poi>,
    #[serde(rename = "Trail", default)]
    pub trail: Vec<Trail>,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub(super) struct Poi {
    #[serde(rename = "@MapID", default)]
    pub map_id: Option<u32>,
    #[serde(rename = "@xpos")]
    pub x: f32,
    #[serde(rename = "@ypos")]
    pub y: f32,
    #[serde(rename = "@zpos")]
    pub z: f32,
    #[serde(rename = "@type")]
    pub id: String,
    #[serde(rename = "@GUID")]
    pub guid: String,
}

impl Poi {
    pub(super) fn from_attrs(attrs: Attributes) -> Result<Self, Error> {
        let mut poi = Self::default();

        for attr in attrs.map(Result::ok).filter_map(identity) {
            println!("attr: {:?}", attr);
            let Ok(key) = String::from_utf8(attr.key.0.to_vec()) else {
                warn!("Key is not UTF-8 encoded: {:?}", attr);
                continue;
            };

            let Ok(value) = String::from_utf8(attr.value.to_vec()) else {
                warn!("Value is not UTF-8 encoded: {:?}", attr);
                continue;
            };

            match key.to_lowercase().as_str() {
                "mapid" => {
                    poi.map_id = attr
                        .value
                        .into_owned()
                        .as_slice()
                        .read_u32::<LittleEndian>()
                        .ok();
                }
                "x" => {
                    poi.x = attr
                        .value
                        .into_owned()
                        .as_slice()
                        .read_f32::<LittleEndian>()
                        .map_err(Error::IoErr)?;
                }
                "y" => {
                    poi.y = attr
                        .value
                        .into_owned()
                        .as_slice()
                        .read_f32::<LittleEndian>()
                        .map_err(Error::IoErr)?;
                }
                "z" => {
                    poi.z = attr
                        .value
                        .into_owned()
                        .as_slice()
                        .read_f32::<LittleEndian>()
                        .map_err(Error::IoErr)?;
                }
                "id" => {
                    poi.id = String::from_utf8(attr.value.to_vec()).map_err(Error::Utf8Error)?;
                }
                "guid" => {
                    poi.guid = String::from_utf8(attr.value.to_vec()).map_err(Error::Utf8Error)?;
                }

                _ => println!("Unknown attribute: {:?}", attr),
            }
        }
        Ok(poi)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub(super) struct Trail {
    #[serde(rename = "@type")]
    pub id: String,
    #[serde(rename = "@trailData")]
    pub trail_data: String,
    #[serde(rename = "@texture")]
    pub texture: String,
    #[serde(rename = "@GUID")]
    pub guid: Option<String>,
}
