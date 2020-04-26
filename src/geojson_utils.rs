use crate::models::{Envelope, Point, Polygon};
use anyhow::{anyhow, Result};
use geojson::{Feature, GeoJson, Value};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use encoding::all::ISO_8859_1;
use encoding::{DecoderTrap, Encoding};
use flate2::read::GzDecoder;

pub fn read_polygons(geojson_filepath: &Path) -> Result<Vec<Polygon>> {
    let features = read_geojson(geojson_filepath)?;
    println!("{:?} yielded {} features", geojson_filepath, features.len());
    Ok(make_polygons(features))
}

// Some of these files are in ISO-8859-1, which we'll have to decode
fn read_geojson(geojson_filepath: &Path) -> Result<Vec<Feature>> {
    let file_gz = File::open(geojson_filepath)?;
    let mut file = GzDecoder::new(file_gz);
    let mut byte_buf = Vec::<u8>::new();
    file.read_to_end(&mut byte_buf)?;
    let json_str = ISO_8859_1.decode(&byte_buf, DecoderTrap::Strict).unwrap();

    let json_val = serde_json::from_str(&json_str)?;
    let us_geojson = GeoJson::from_json_value(json_val)?;
    if let GeoJson::FeatureCollection(fc) = us_geojson {
        Ok(fc.features)
    } else {
        Err(anyhow!("Supplied geojson is not a feature collection"))
    }
}

fn get_json_string(val: serde_json::Value) -> String {
    match val {
        serde_json::Value::String(s) => s.trim().to_string(),
        _ => panic!("Unexpected value type"),
    }
}

fn make_polygons(features: Vec<Feature>) -> Vec<Polygon> {
    features
        .into_iter()
        .filter_map(|feature| {
            let mut properties = feature.properties?;
            let state_id = properties.remove("STATE").map(get_json_string)?;
            let maybe_county_id = properties.remove("COUNTY").map(get_json_string);
            let id = match &maybe_county_id {
                None => state_id,
                Some(county_id) => format!("{}-{}", state_id, county_id),
            };
            let boundaries: Vec<Vec<Vec<f64>>> = match feature.geometry?.value {
                Value::Polygon(mut loops) => vec![loops.remove(0)],
                Value::MultiPolygon(loops_list) => loops_list
                    .into_iter()
                    .map(|mut loops| loops.remove(0))
                    .collect(),
                _ => panic!("Unexpected geometry type"),
            };
            let polys: Vec<Polygon> = boundaries
                .iter()
                .map(convert_to_points)
                .map(|lp| Polygon {
                    id: id.clone(),
                    envelope: Envelope::of(&lp),
                    coords: lp,
                })
                .collect();

            Some(polys)
        })
        .flatten()
        .collect()
}

fn convert_to_points(coords: &Vec<Vec<f64>>) -> Vec<Point> {
    coords.iter().map(|c| Point { x: c[0], y: c[1] }).collect()
}
