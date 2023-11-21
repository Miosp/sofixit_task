use serde::{Serialize, Deserialize};

use crate::data_gen::FakeData;

#[derive(Deserialize, Serialize)]
pub struct CSVData {
    pub r#type: String,
    pub _id: u32,
    pub name: String,
    pub latitude: String,
    pub longitude: String,
}

impl From<FakeData> for CSVData {
    fn from(data: FakeData) -> Self {
        CSVData {
            r#type: data.r#type,
            _id: data._id,
            name: data.name,
            latitude: data.geo_position.latitude,
            longitude: data.geo_position.longitude,
        }
    }
}