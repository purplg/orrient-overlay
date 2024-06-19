use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct OverlayData {
    #[serde(rename = "MarkerCategory", default)]
    pub categories: Vec<MarkerCategory>,
    #[serde(rename = "POIs", default)]
    pub pois: Vec<POIs>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MarkerCategory {
    #[serde(rename = "@name", alias = "@bh-name", default)]
    pub name: String,
    #[serde(rename = "@DisplayName", alias = "@bh-DisplayName")]
    pub display_name: String,
    #[serde(rename = "@IsSeparator")]
    pub is_separator: Option<bool>,
    #[serde(rename = "MarkerCategory")]
    #[serde(default)]
    pub categories: Vec<MarkerCategory>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct POIs {
    #[serde(rename = "POI", default)]
    pub poi: Vec<POI>,
    #[serde(rename = "Trail", default)]
    pub trail: Vec<Trail>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct POI {
    #[serde(rename = "@MapId", skip_serializing_if = "Option::is_none", default)]
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

#[derive(Serialize, Deserialize, Debug)]
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
        let iter = std::fs::read_dir("~/.config/orrient/markers").unwrap();
        for path in iter
            .filter_map(|file| file.ok().map(|file| file.path()))
            .filter(|file| file.is_file())
            .filter(|file| file.extension().map(|ext| ext == "xml").unwrap_or_default())
        {
            println!("Testing: {:?}", path);
            let a = std::fs::read_to_string(path).unwrap();
            let _de: OverlayData = quick_xml::de::from_str(&a).unwrap();
        }
    }
}
