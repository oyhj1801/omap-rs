use crate::{
    map_coord::MapCoord, map_object::MapObjectTrait, symbol::PointSymbol, OmapResult, Scale,
    TagTrait,
};
use geo_types::Point;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
};

#[derive(Debug, Clone)]
pub struct PointObject {
    pub point: Point,
    pub symbol: PointSymbol,
    pub rotation: f64,
    pub tags: HashMap<String, String>,
}

impl PointObject {
    pub fn from_point(point: Point, symbol: PointSymbol, rotation: f64) -> Self {
        Self {
            point,
            symbol,
            rotation,
            tags: HashMap::new(),
        }
    }
}

impl TagTrait for PointObject {
    fn add_tag(&mut self, k: impl Into<String>, v: impl Into<String>) {
        self.tags.insert(k.into(), v.into());
    }
}

impl MapObjectTrait for PointObject {
    fn write_to_map(
        self,
        f: &mut BufWriter<File>,
        _as_bezier: Option<f64>,
        scale: Scale,
        grivation: f64,
        combined_scale_factor: f64,
    ) -> OmapResult<()> {
        f.write_all(
            format!(
                "<object type=\"0\" symbol=\"{}\" rotation=\"{}\">",
                self.symbol.id(),
                self.rotation
            )
            .as_bytes(),
        )?;
        self.write_tags(f)?;
        self.write_coords(f, None, scale, grivation, combined_scale_factor)?;
        f.write_all(b"</object>\n")?;
        Ok(())
    }

    fn write_coords(
        self,
        f: &mut BufWriter<File>,
        _as_bezier: Option<f64>,
        scale: Scale,
        grivation: f64,
        combined_scale_factor: f64,
    ) -> OmapResult<()> {
        let c = self
            .point
            .0
            .to_map_coordinates(scale, grivation, combined_scale_factor)?;
        f.write_all(format!("<coords count=\"1\">{} {};</coords>", c.0, c.1).as_bytes())?;
        Ok(())
    }

    fn write_tags(&self, f: &mut BufWriter<File>) -> OmapResult<()> {
        if self.tags.is_empty() {
            return Ok(());
        }

        f.write_all(b"<tags>")?;
        for (key, val) in self.tags.iter() {
            f.write_all(format!("<t k=\"{key}\">{val}</t>").as_bytes())?;
        }
        f.write_all(b"</tags>")?;
        Ok(())
    }
}
