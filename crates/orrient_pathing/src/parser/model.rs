use std::{borrow::Cow, convert::identity, str::FromStr};

use anyhow::{anyhow, Result};
use bevy::{log::warn, math::Vec3};
use quick_xml::events::attributes::Attributes;
use typed_path::{Utf8PathBuf, Utf8UnixEncoding, Utf8WindowsPathBuf};

use super::pack::MarkerId;

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

#[derive(Clone, Debug)]
pub struct Poi {
    // type
    pub id: MarkerId,
    // MapID
    pub map_id: Option<u32>,
    // xpos, ypos, zpos
    pub position: Option<Vec3>,
    // iconFile
    pub icon_file: Option<Utf8PathBuf<Utf8UnixEncoding>>,
}

impl Poi {
    pub(super) fn from_attrs(attrs: Attributes) -> Result<Self> {
        let mut map_id: Option<u32> = None;
        let mut x: Option<f32> = None;
        let mut y: Option<f32> = None;
        let mut z: Option<f32> = None;
        let mut id: Option<String> = None;
        let mut icon_file: Option<Utf8PathBuf<Utf8UnixEncoding>> = None;

        for attr in attrs.map(Result::ok).flatten() {
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
                "iconfile" => {
                    if let Ok(path) = Utf8WindowsPathBuf::from_str(&value) {
                        icon_file = Some(path.with_unix_encoding().to_path_buf());
                    } else {
                        warn!("Icon path is corrupt: {:?}", attr)
                    }
                }

                _ => {}
            }
        }

        // Catch if only a position is only partially defined.
        // `num_coords` should only be 3 or 0. If it's anything else,
        // then a xpos, ypos, zpos is either duplicate or missing.
        let num_coords = [x, y, z].into_iter().filter_map(identity).count();
        let position = if num_coords == 3 {
            Some(Vec3::new(x.unwrap(), y.unwrap(), z.unwrap()))
        } else {
            if num_coords != 0 {
                if let Some(ref id) = id {
                    warn!("POI has an invalid position: {id}");
                } else {
                    warn!("POI has an invalid position.");
                }
            }
            None
        };

        Ok(Poi {
            id: MarkerId(id.ok_or(anyhow!("POI missing field `poi.type`."))?.into()),
            map_id,
            position,
            icon_file,
        })
    }
}

#[derive(Clone, Debug)]
pub(super) struct TrailXml {
    // type
    pub id: Cow<'static, str>,
    // trailData
    pub trail_file: String,
    // texture
    pub texture_file: String,
}

impl TrailXml {
    pub(super) fn from_attrs(attrs: Attributes) -> Result<Self> {
        let mut id: Option<String> = None;
        let mut trail_file: Option<String> = None;
        let mut texture_file: Option<String> = None;

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
                "type" => {
                    id = value.parse().ok();
                }
                "traildata" => {
                    trail_file = value.parse().ok();
                }
                "texture" => {
                    texture_file = value.parse().ok();
                }
                _ => {}
            }
        }

        Ok(Self {
            id: id.ok_or(anyhow!("Trail missing field `type`."))?.into(),
            trail_file: trail_file.ok_or(anyhow!("Trail missing field `trailData`."))?,
            texture_file: texture_file.ok_or(anyhow!("Trail missing field `texture`."))?,
        })
    }
}
