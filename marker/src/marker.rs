use serde::{Deserialize, Deserializer};

#[derive(Clone, Deserialize, Debug)]
pub struct OverlayData {
    #[serde(rename = "MarkerCategory", default)]
    pub categories: Vec<MarkerCategory>,
    #[serde(rename = "POIs", default)]
    pub pois: Vec<POIs>,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct MarkerCategory {
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
    pub fn id(&self) -> String {
        self._name
            .clone()
            .or_else(|| self.bh_name.clone())
            .expect("No category name")
            .clone()
    }

    pub fn display_name(&self) -> String {
        self._display_name
            .clone()
            .or_else(|| self.bh_display_name.clone())
            .expect("No category name")
            .clone()
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct POIs {
    #[serde(rename = "POI", default)]
    pub poi: Vec<POI>,
    #[serde(rename = "Trail", default)]
    pub trail: Vec<Trail>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct POI {
    #[serde(rename = "@MapId", default)]
    pub map_id: Option<usize>,
    #[serde(rename = "@xpos")]
    pub x: f32,
    #[serde(rename = "@ypos")]
    pub y: f32,
    #[serde(rename = "@zpos")]
    pub z: f32,
    #[serde(rename = "@type")]
    pub kind: String,
    #[serde(rename = "@GUID")]
    pub guid: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Trail {
    #[serde(rename = "@type")]
    pub kind: String,
    #[serde(rename = "@trailData")]
    pub trail_data: String,
    #[serde(rename = "@texture")]
    pub texture: String,
    #[serde(rename = "@GUID")]
    pub guid: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markers() {
        let iter = std::fs::read_dir("/home/purplg/.config/orrient/markers").unwrap();
        for path in iter
            .filter_map(|file| file.ok().map(|file| file.path()))
            .filter(|file| file.is_file())
            .filter(|file| file.extension().map(|ext| ext == "xml").unwrap_or_default())
        {
            println!("Testing: {:?}", path);
            let data = std::fs::read_to_string(path).unwrap();
            let overlay: OverlayData = quick_xml::de::from_str(&data).unwrap();
        }
    }
}
