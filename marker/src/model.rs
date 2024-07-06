#![allow(unused)]

use std::convert::identity;

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
        let mut map_id: Option<u32> = None;
        let mut x: Option<f32> = None;
        let mut y: Option<f32> = None;
        let mut z: Option<f32> = None;
        let mut id: Option<String> = None;
        let mut guid: Option<String> = None;

        for attr in attrs.map(Result::ok).filter_map(identity) {
            let Ok(key) = String::from_utf8(attr.key.0.to_vec()) else {
                warn!("Key is not UTF-8 encoded: {:?}", attr);
                continue;
            };

            let Ok(value) = String::from_utf8(attr.value.trim_ascii().to_vec()) else {
                warn!("Value is not UTF-8 encoded: {:?}", attr);
                continue;
            };

            match key.to_lowercase().as_str() {
                "mapid" => {
                    map_id = value.parse().ok();
                }
                "xpos" => {
                    x = value.parse().ok();
                }
                "ypos" => {
                    y = value.parse().ok();
                }
                "zpos" => {
                    z = value.parse().ok();
                }
                "type" => {
                    id = Some(value);
                }
                "guid" => {
                    guid = Some(value);
                }

                _ => {}
            }
        }
        Ok(Poi {
            map_id,
            x: x.ok_or(Error::FieldErr {
                field: "poi.x".into(),
                message: "POI Missing X position".into(),
            })?,
            y: y.ok_or(Error::FieldErr {
                field: "poi.y".into(),
                message: "POI Missing Y position".into(),
            })?,
            z: z.ok_or(Error::FieldErr {
                field: "poi.z".into(),
                message: "POI Missing Z position".into(),
            })?,
            id: id.ok_or(Error::FieldErr {
                field: "poi.id".into(),
                message: "POI Missing id".into(),
            })?,
            guid: guid.ok_or(Error::FieldErr {
                field: "poi.guid".into(),
                message: "POI Missing guid".into(),
            })?,
        })
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
