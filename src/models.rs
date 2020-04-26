use serde::Deserialize;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct PopCenter {
    #[serde(rename = "longitude")]
    pub x: f64,
    #[serde(rename = "latitude")]
    pub y: f64,
    #[serde(rename = "population_2020")]
    pub population: f64,
}

impl PopCenter {
    pub fn as_point(&self) -> Point {
        Point {
            x: self.x,
            y: self.y,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Envelope {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl Envelope {
    pub fn empty() -> Self {
        Envelope {
            min_x: std::f64::INFINITY,
            min_y: std::f64::INFINITY,
            max_x: std::f64::NEG_INFINITY,
            max_y: std::f64::NEG_INFINITY,
        }
    }

    pub fn of(points: &[Point]) -> Self {
        let mut min_x = std::f64::INFINITY;
        let mut min_y = std::f64::INFINITY;
        let mut max_x = std::f64::NEG_INFINITY;
        let mut max_y = std::f64::NEG_INFINITY;

        points.iter().for_each(|p| {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        });

        Envelope {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn expand(&mut self, other: Envelope) {
        self.min_x = self.min_x.min(other.min_x);
        self.min_y = self.min_x.min(other.min_x);
        self.max_x = self.max_x.max(other.max_x);
        self.max_y = self.max_y.max(other.max_y);
    }

    pub fn contains(&self, point: Point) -> bool {
        self.min_x <= point.x
            && point.x <= self.max_x
            && self.min_y <= point.y
            && point.y <= self.max_y
    }
}

#[derive(Debug, Clone)]
pub struct Polygon {
    pub id: String,
    pub envelope: Envelope,
    pub coords: Vec<Point>,
}

impl Polygon {
    pub fn contains(&self, point: Point) -> bool {
        point_in_polygon_check(point, &self.coords)
    }
}

fn point_in_polygon_check(point: Point, coords: &[Point]) -> bool {
    // adapted from http://geomalgorithms.com/a03-_inclusion.html
    // Assumes polygon is "valid" and "simple", as per OGC/ISO
    assert!(coords.len() > 3);
    assert_eq!(coords.first(), coords.last());

    let mut wn = 0;
    for (start, end) in coords.iter().zip(coords.iter().skip(1)) {
        if point.x > start.x && point.x > end.x {
            // Point is to the right
            continue;
        }

        // Calculate the two halves of the cross-product (= lx - rx)
        let lx = (end.x - start.x) * (point.y - start.y);
        let rx = (end.y - start.y) * (point.x - start.x);

        if start.y <= point.y {
            // Upward crossing
            if end.y > point.y && lx > rx {
                wn += 1;
            }
        } else {
            // Downward crossing
            if end.y <= point.y && lx < rx {
                wn -= 1;
            }
        }
    }

    wn != 0
}
