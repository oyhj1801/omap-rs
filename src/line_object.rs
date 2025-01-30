use geo_types::LineString;

use polyline2bezier::{BezierSegmentType, BezierString};

use crate::{map_coord::MapCoord, map_object::MapObjectTrait, OmapResult, Scale, Symbol, Tag};

use std::{
    fs::File,
    io::{BufWriter, Write},
};

pub struct LineObject {
    symbol: Symbol,
    tags: Vec<Tag>,
}

impl LineObject {
    pub fn from_symbol(symbol: Symbol) -> Self {
        Self {
            symbol,
            tags: vec![],
        }
    }

    fn write_polyline(
        self,
        f: &mut BufWriter<File>,
        scale: Scale,
        grivation: f64,
        combined_scale_factor: f64,
    ) -> OmapResult<()> {
        let num_coords = self.symbol.num_coords();

        let coordinates = LineString::try_from(self.symbol)?;

        f.write_all(format!("<coords count=\"{num_coords}\">").as_bytes())?;

        let mut coord_iter = coordinates.coords();
        let mut i = 0;
        while i < num_coords - 1 {
            let c = coord_iter.next().unwrap().to_map_coordinates(
                scale,
                grivation,
                combined_scale_factor,
            )?;
            f.write_all(format!("{} {};", c.0, c.1).as_bytes())?;

            i += 1;
        }
        let c = coord_iter.next().unwrap().to_map_coordinates(
            scale,
            grivation,
            combined_scale_factor,
        )?;
        if coordinates.is_closed() {
            f.write_all(format!("{} {} 18;", c.0, c.1).as_bytes())?;
        } else {
            f.write_all(format!("{} {};", c.0, c.1).as_bytes())?;
        }

        f.write_all(b"</coords>")?;
        Ok(())
    }

    fn write_bezier(
        self,
        f: &mut BufWriter<File>,
        error: f64,
        scale: Scale,
        grivation: f64,
        combined_scale_factor: f64,
    ) -> OmapResult<()> {
        let coordinates = LineString::try_from(self.symbol)?;

        let bezier = BezierString::from_polyline(&coordinates, error);

        let num_coords = bezier.num_points();
        let num_segments = bezier.0.len();
        f.write_all(format!("<coords count=\"{num_coords}\">").as_bytes())?;

        let mut bez_iterator = bezier.0.into_iter();
        let mut i = 0;
        while i < num_segments - 1 {
            let segment = bez_iterator.next().unwrap();
            match segment.line_type() {
                BezierSegmentType::Polyline => {
                    let c =
                        segment
                            .0
                             .0
                            .to_map_coordinates(scale, grivation, combined_scale_factor)?;

                    f.write_all(format!("{} {};", c.0, c.1).as_bytes())?;
                }
                BezierSegmentType::Bezier => {
                    let c =
                        segment
                            .0
                             .0
                            .to_map_coordinates(scale, grivation, combined_scale_factor)?;
                    let h1 = segment.0 .1.unwrap().to_map_coordinates(
                        scale,
                        grivation,
                        combined_scale_factor,
                    )?;
                    let h2 = segment.0 .2.unwrap().to_map_coordinates(
                        scale,
                        grivation,
                        combined_scale_factor,
                    )?;
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
                let c1 = final_segment.0 .0.to_map_coordinates(
                    scale,
                    grivation,
                    combined_scale_factor,
                )?;
                let c2 = final_segment.0 .3.to_map_coordinates(
                    scale,
                    grivation,
                    combined_scale_factor,
                )?;

                if coordinates.is_closed() {
                    f.write_all(format!("{} {};{} {} 18;", c1.0, c1.1, c2.0, c2.1).as_bytes())?;
                } else {
                    f.write_all(format!("{} {};{} {};", c1.0, c1.1, c2.0, c2.1).as_bytes())?;
                }
            }
            BezierSegmentType::Bezier => {
                let c1 = final_segment.0 .0.to_map_coordinates(
                    scale,
                    grivation,
                    combined_scale_factor,
                )?;
                let h1 = final_segment.0 .1.unwrap().to_map_coordinates(
                    scale,
                    grivation,
                    combined_scale_factor,
                )?;
                let h2 = final_segment.0 .2.unwrap().to_map_coordinates(
                    scale,
                    grivation,
                    combined_scale_factor,
                )?;
                let c2 = final_segment.0 .3.to_map_coordinates(
                    scale,
                    grivation,
                    combined_scale_factor,
                )?;

                if coordinates.is_closed() {
                    f.write_all(
                        format!(
                            "{} {} 1;{} {};{} {};{} {} 18;",
                            c1.0, c1.1, h1.0, h1.1, h2.0, h2.1, c2.0, c2.1
                        )
                        .as_bytes(),
                    )?;
                } else {
                    f.write_all(
                        format!(
                            "{} {} 1;{} {};{} {};{} {};",
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

impl MapObjectTrait for LineObject {
    fn add_tag(&mut self, k: &str, v: &str) {
        self.tags.push(Tag::new(k, v));
    }

    fn write_to_map(
        self,
        f: &mut BufWriter<File>,
        bez_error: Option<f64>,
        scale: Scale,
        grivation: f64,
        combined_scale_factor: f64,
    ) -> OmapResult<()> {
        f.write_all(format!("<object type=\"1\" symbol=\"{}\">", self.symbol.id()).as_bytes())?;
        self.write_tags(f)?;
        self.write_coords(f, bez_error, scale, grivation, combined_scale_factor)?;
        f.write_all(b"</object>\n")?;
        Ok(())
    }

    fn write_coords(
        self,
        f: &mut BufWriter<File>,
        bez_error: Option<f64>,
        scale: Scale,
        grivation: f64,
        combined_scale_factor: f64,
    ) -> OmapResult<()> {
        if let Some(error) = bez_error {
            self.write_bezier(f, error, scale, grivation, combined_scale_factor)
        } else {
            self.write_polyline(f, scale, grivation, combined_scale_factor)
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
