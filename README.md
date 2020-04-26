# Spatial Join Performance

This code compares performing a spatial join of a population dataset with
polygons of counties in the USA in four ways:

1. A naive double loop
2. Using a cheap county envelope pre-check
3. Checking a state's envelope first, then checking the state's counties.
4. Using an RTree of the county polygons.

On my machine, (1) took 652.1 seconds, (2) took 13.8s, (3) took 3.4s, and (4)
took 1.3s.

These results are not deeply rigorous, nor are the algorithms particularly
optimized. Additionally, only the outer shells of polygons are used -- holes
are ignored completely. They are only intended to get order-of-magnitude results.

## To run the performance measurements

1. [Install Rust](https://www.rust-lang.org/tools/install)
2. Clone this repo.
3. In the root directory, run `cargo build --release && target/release/presto_spatial_join_blog`

The brute force calculation can take over 10 minutes, so watch a video from
[Lessons from the Screenplay](https://www.lessonsfromthescreenplay.com/).

## Acknowledgements

Population centers come from Facebook's
[Population Density Maps](https://dataforgood.fb.com/tools/population-density-maps/).

County and State geojson files come from [Eric Celeste](https://eric.clst.org/tech/usgeojson/),
who sourced the data from the [US Census Bureau](https://www.census.gov/geographies/mapping-files/time-series/geo/carto-boundary-file.html).
