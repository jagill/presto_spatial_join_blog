mod geojson_utils;
mod models;
mod rtree;
mod utils;

use anyhow::{anyhow, Result};
use geojson_utils::read_polygons;
use models::{Envelope, Polygon, PopCenter};
use rtree::{build_rtree, RTree, AABB};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use utils::{build_polygon_envelopes, build_state_id_to_counties, read_population};

fn main() -> Result<()> {
    let data_dir = Path::new("data");
    let population_filepath = data_dir.join("us_popdens_z14.csv.gz");
    let counties_geojson_filepath = data_dir.join("gz_2010_us_050_00_5m.json.gz");
    let states_geojson_filepath = data_dir.join("gz_2010_us_040_00_5m.json.gz");

    let pop_centers = read_population(&population_filepath)?;
    println!("Found {} population centers.", pop_centers.len());
    let counties = read_polygons(&counties_geojson_filepath)?;
    println!("Found {} county polygons.", counties.len());

    ////////
    // Double loop
    let now = Instant::now();
    let pop_count = do_double_loop(&pop_centers, &counties);
    println!(
        "Calculating brute force took {}ms, with {} counties populated",
        now.elapsed().as_millis(),
        pop_count.len()
    );

    ////////
    // Double loop with county envelopes
    let now = Instant::now();
    let pop_count_envelope = do_double_loop_with_envelope(&pop_centers, &counties);
    println!(
        "Calculating with county envelopes with envelope took {}ms, with {} counties populated",
        now.elapsed().as_millis(),
        pop_count_envelope.len()
    );
    check_pop_counts(&pop_count, &pop_count_envelope)?;

    ////////
    // Loop with county and state envelopes
    let state_envelopes = build_polygon_envelopes(&read_polygons(&states_geojson_filepath)?);
    println!("Found {} state envelopes.", state_envelopes.len());

    let now = Instant::now();
    let pop_count_state_envelopes =
        do_loop_with_state_envelopes(&pop_centers, &counties, &state_envelopes);
    println!(
        "Calculating with state and county envelopes took {}ms, with {} counties populated",
        now.elapsed().as_millis(),
        pop_count_state_envelopes.len()
    );
    check_pop_counts(&pop_count, &pop_count_state_envelopes)?;

    ////////
    // Rtree
    let now = Instant::now();
    let rtree = build_rtree(&counties);
    let pop_count_rtree = do_rtree(&pop_centers, &rtree);
    println!(
        "Calculating with rtree took {}ms, with {} counties populated",
        now.elapsed().as_millis(),
        pop_count_rtree.len()
    );
    check_pop_counts(&pop_count, &pop_count_rtree)?;

    Ok(())
}

fn do_double_loop(centers: &[PopCenter], counties: &[Polygon]) -> HashMap<String, f64> {
    let mut map: HashMap<String, f64> = HashMap::new();
    for center in centers {
        let point = center.as_point();
        for county in counties {
            if county.contains(point) {
                let value = map.entry(county.id.clone()).or_insert(0.);
                *value += center.population;
            }
        }
    }

    map
}

fn do_double_loop_with_envelope(
    centers: &[PopCenter],
    counties: &[Polygon],
) -> HashMap<String, f64> {
    let mut map: HashMap<String, f64> = HashMap::new();
    for center in centers {
        let point = center.as_point();
        for county in counties {
            if county.envelope.contains(point) && county.contains(point) {
                let value = map.entry(county.id.clone()).or_insert(0.);
                *value += center.population;
            }
        }
    }

    map
}

fn do_loop_with_state_envelopes(
    centers: &[PopCenter],
    counties: &[Polygon],
    state_envelopes: &HashMap<String, Envelope>,
) -> HashMap<String, f64> {
    let mut map: HashMap<String, f64> = HashMap::new();

    let state_id_to_counties = build_state_id_to_counties(&counties).unwrap();
    for center in centers {
        let point = center.as_point();
        for (state_id, state_envelope) in state_envelopes.iter() {
            if !state_envelope.contains(point) {
                continue;
            }
            let state_counties = state_id_to_counties.get(state_id).unwrap();
            for county in state_counties {
                if county.envelope.contains(point) && county.contains(point) {
                    let value = map.entry(county.id.clone()).or_insert(0.);
                    *value += center.population;
                }
            }
        }
    }

    map
}

fn do_rtree(centers: &[PopCenter], rtree: &RTree<Polygon>) -> HashMap<String, f64> {
    let mut map: HashMap<String, f64> = HashMap::new();
    for center in centers {
        let point = center.as_point();
        let env = AABB::from_point([point.x, point.y]);
        for county in rtree.locate_in_envelope_intersecting(&env) {
            if county.contains(point) {
                let value = map.entry(county.id.clone()).or_insert(0.);
                *value += center.population;
            }
        }
    }

    map
}

fn check_pop_counts(
    base_pop: &HashMap<String, f64>,
    compare_pop: &HashMap<String, f64>,
) -> Result<()> {
    if base_pop == compare_pop {
        println!("Population counts are the same, with and without envelope.");
        Ok(())
    } else {
        Err(anyhow!("Population counts differ!"))
    }
}
