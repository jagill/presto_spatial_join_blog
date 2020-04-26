use crate::models::{Envelope, Polygon, PopCenter};
use anyhow::{anyhow, Result};
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

pub fn read_population(population_filepath: &Path) -> Result<Vec<PopCenter>, csv::Error> {
    let file_gz = File::open(population_filepath)?;
    let file = GzDecoder::new(file_gz);
    let mut reader = csv::Reader::from_reader(file);
    reader.deserialize().collect()
}

pub fn find_state_id(county_id: &str) -> Option<String> {
    county_id.split('-').next().map(str::to_string)
}

pub fn build_polygon_envelopes(polygons: &[Polygon]) -> HashMap<String, Envelope> {
    let mut envelopes: HashMap<String, Envelope> = HashMap::with_capacity(polygons.len());
    for polygon in polygons {
        let envelope = envelopes
            .entry(polygon.id.clone())
            .or_insert_with(Envelope::empty);
        envelope.expand(polygon.envelope);
    }

    envelopes
}

pub fn build_state_id_to_counties(counties: &[Polygon]) -> Result<HashMap<String, Vec<&Polygon>>> {
    let mut state_to_counties: HashMap<String, Vec<&Polygon>> =
        HashMap::with_capacity(counties.len());
    for county in counties {
        let state_id = find_state_id(&county.id).ok_or_else(|| anyhow!("Missing state id"))?;
        let state_counties = state_to_counties.entry(state_id).or_insert_with(Vec::new);
        state_counties.push(county);
    }
    Ok(state_to_counties)
}
