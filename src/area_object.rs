use geo_types::Polygon;

use polyline2bezier::{BezierSegmentType, BezierString};

use crate::{map_geo_traits::MapCoord, map_object::MapObjectTrait, OmapResult, Scale, Symbol, Tag};

use std::{
    fs::File,
    io::{BufWriter, Write},
};

pub struct AreaObject {
    symbol: Symbol,
    tags: Vec<Tag>,
}

impl AreaObject {
    pub fn from_polygon(symbol: Symbol) -> Self {
        Self {
            symbol,
            tags: vec![],
        }
    }

    fn write_polyline(self, f: &mut BufWriter<File>, scale: Scale) -> OmapResult<()> {
        let coordinates = Polygon::try_from(self.symbol)?;

        let mut num_coords = coordinates.exterior().0.len();
        let boundary_length = num_coords;

        for hole in coordinates.interiors().iter() {
            num_coords += hole.0.len();
        }

        f.write_all(format!("<coords count=\"{}\">", num_coords).as_bytes())?;

        let mut ext_iter = coordinates.exterior().coords();
        let mut i = 0;

        while i < boundary_length - 1 {
            let c = ext_iter.next().unwrap().to_map_coordinates(scale)?;
            f.write_all(format!("{} {};", c.0, c.1).as_bytes())?;
            i += 1;
        }
        let c = ext_iter.next().unwrap().to_map_coordinates(scale)?;
        f.write_all(format!("{} {} 18;", c.0, c.1).as_bytes())?;

        for hole in coordinates.interiors().iter() {
            let hole_length = hole.0.len();

            let mut int_iter = hole.coords();
            let mut i = 0;

            while i < hole_length - 1 {
                let c = int_iter.next().unwrap().to_map_coordinates(scale)?;
                f.write_all(format!("{} {};", c.0, c.1).as_bytes())?;

                i += 1;
            }
            let c = int_iter.next().unwrap().to_map_coordinates(scale)?;
            f.write_all(format!("{} {} 18;", c.0, c.1).as_bytes())?;
        }
        f.write_all(b"</coords>")?;

        Ok(())
    }

    fn write_bezier(self, f: &mut BufWriter<File>, error: f64, scale: Scale) -> OmapResult<()> {
        let coordinates = Polygon::try_from(self.symbol)?;

        let mut beziers = Vec::with_capacity(coordinates.num_rings());
        beziers.push(BezierString::from_polyline(coordinates.exterior(), error));
        for hole in coordinates.interiors() {
            beziers.push(BezierString::from_polyline(hole, error));
        }
        let mut num_coords = 0;
        for b in beziers.iter() {
            num_coords += b.num_points();
        }

        f.write_all(format!("<coords count=\"{num_coords}\">").as_bytes())?;

        for bezier in beziers {
            let num_segments = bezier.0.len();

            let mut bez_iterator = bezier.0.into_iter();
            let mut i = 0;
            while i < num_segments - 1 {
                let segment = bez_iterator.next().unwrap();
                match segment.line_type() {
                    BezierSegmentType::Polyline => {
                        let c = segment.0 .0.to_map_coordinates(scale)?;

                        f.write_all(format!("{} {};", c.0, c.1).as_bytes())?;
                    }
                    BezierSegmentType::Bezier => {
                        let c = segment.0 .0.to_map_coordinates(scale)?;
                        let h1 = segment.0 .1.unwrap().to_map_coordinates(scale)?;
                        let h2 = segment.0 .2.unwrap().to_map_coordinates(scale)?;
                        f.write_all(
                            format!("{} {} 1;{} {};{} {};", c.0, c.1, h1.0, h1.1, h2.0, h2.1)
                                .as_bytes(),
                        )?;
                    }
                }
                i += 1;
            }
            // finish with the last segment of the curve
            let final_segment = bez_iterator.next().unwrap();
            match final_segment.line_type() {
                BezierSegmentType::Polyline => {
                    let c1 = final_segment.0 .0.to_map_coordinates(scale)?;
                    let c2 = final_segment.0 .3.to_map_coordinates(scale)?;

                    f.write_all(format!("{} {};{} {} 18;", c1.0, c1.1, c2.0, c2.1).as_bytes())?;
                }
                BezierSegmentType::Bezier => {
                    let c1 = final_segment.0 .0.to_map_coordinates(scale)?;
                    let h1 = final_segment.0 .1.unwrap().to_map_coordinates(scale)?;
                    let h2 = final_segment.0 .2.unwrap().to_map_coordinates(scale)?;
                    let c2 = final_segment.0 .3.to_map_coordinates(scale)?;

                    f.write_all(
                        format!(
                            "{} {} 1;{} {};{} {};{} {} 18;",
                            c1.0, c1.1, h1.0, h1.1, h2.0, h2.1, c2.0, c2.1
                        )
                        .as_bytes(),
                    )?;
                }
            }
        }
        f.write_all(b"</coords>")?;
        Ok(())
    }
}

impl MapObjectTrait for AreaObject {
    fn add_tag(&mut self, k: &str, v: &str) {
        self.tags.push(Tag::new(k, v));
    }

    fn write_to_map(
        self,
        f: &mut BufWriter<File>,
        bez_error: Option<f64>,
        scale: Scale,
    ) -> OmapResult<()> {
        f.write_all(format!("<object type=\"1\" symbol=\"{}\">", self.symbol.id()).as_bytes())?;
        self.write_tags(f)?;
        self.write_coords(f, bez_error, scale)?;
        f.write_all(b"</object>\n")?;
        Ok(())
    }

    fn write_coords(
        self,
        f: &mut BufWriter<File>,
        bez_error: Option<f64>,
        scale: Scale,
    ) -> OmapResult<()> {
        if let Some(error) = bez_error {
            self.write_bezier(f, error, scale)
        } else {
            self.write_polyline(f, scale)
        }
    }

    fn write_tags(&self, f: &mut BufWriter<File>) -> OmapResult<()> {
        if self.tags.is_empty() {
            return Ok(());
        }

        f.write_all(b"<tags>")?;
        for tag in self.tags.iter() {
            f.write_all(tag.to_string().as_bytes())?;
        }
        f.write_all(b"</tags>")?;
        Ok(())
    }
}
