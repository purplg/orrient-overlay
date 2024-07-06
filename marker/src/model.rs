#![allow(unused)]

use std::convert::identity;

use byteorder::{LittleEndian, ReadBytesExt};
use log::warn;
use quick_xml::{events::attributes::Attributes, name::QName};

use super::Error;

#[derive(Clone, Debug, Default)]
pub(super) struct MarkerCategory {
    // Name
    pub id: String,
    // DisplayName
    pub display_name: String,
    // IsSeparator
    pub is_separator: bool,
    // fadeNear
    pub fade_near: Option<f32>,
    // fadeFar
    pub fade_far: Option<f32>,
    // iconFile
    pub icon_path: Option<String>,
    // iconSize
    pub icon_size: Option<f32>,
    // mapDisplaySize
    pub map_display_size: Option<f32>,
    // inGameVisibility
    pub show_on_ingame: bool,
    // mapVisibility
    pub show_on_map: bool,
    // miniMapVisibility
    pub show_on_minimap: bool,
    // heightOffset
    pub height_offset: Option<f32>,
    // minSize
    pub min_size: Option<f32>,
    // achievementId
    pub achievement_id: Option<u32>,
    // achievementBit
    pub achievement_bit: Option<u8>,
    // bounce
    pub bounce: Option<String>,
    // bounce-height
    pub bounce_height: Option<f32>,
    // autotrigger
    pub autotrigger: bool,
    // triggerrange
    pub triggerrange: Option<f32>,
    // tip-name
    pub tip_name: Option<String>,
    // tip-description
    pub tip_description: Option<String>,
    // behavior
    pub behavior: Option<u8>,
    // copy
    pub copy: Option<String>,
    // copy-message
    pub copy_message: Option<String>,
    // resetLength
    pub reset_length: Option<f32>,
    // toggleCategory
    pub toggle_category: Option<String>,
    // profession
    pub profession: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub(super) struct Poi {
    // MapID
    pub map_id: Option<u32>,
    // xpos
    pub x: f32,
    // ypos
    pub y: f32,
    // zpos
    pub z: f32,
    // type
    pub id: String,
    // GUID
    pub guid: String,
}

impl Poi {
    pub(super) fn from_attrs(attrs: Attributes) -> Result<Self, Error> {
        let mut poi = Self::default();

        for attr in attrs.map(Result::ok).filter_map(identity) {
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
                "type" => {
                    poi.id = String::from_utf8(attr.value.to_vec()).map_err(Error::Utf8Error)?;
                }
                "guid" => {
                    poi.guid = String::from_utf8(attr.value.to_vec()).map_err(Error::Utf8Error)?;
                }

                _ => {}
            }
        }
        Ok(poi)
    }
}

#[derive(Clone, Debug)]
pub(super) struct Trail {
    // type
    pub id: String,
    // trailData
    pub trail_data: String,
    // texture
    pub texture: String,
    // GUID
    pub guid: Option<String>,
}
