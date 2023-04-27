use bevy::{
    asset::Handle,
    math::{Mat4, Vec2},
    reflect::TypeUuid,
    render::{color::Color, mesh::Mesh},
    transform::components::Transform,
};
use copyless::VecHelper;
use lyon_geom::euclid::default::Transform2D;
use lyon_path::PathEvent;
use lyon_tessellation::math::Point;
use svgtypes::ViewBox;
use vello_svg::usvg;

use super::Convert;

/// A loaded and deserialized SVG file.
#[derive(Debug, TypeUuid)]
#[uuid = "d2c5985d-e221-4257-9e3b-ff0fb87e28ba"]
pub struct Svg {
    /// The name of the file.
    pub name: String,
    /// Size of the SVG.
    pub size: Vec2,
    /// ViewBox of the SVG.
    pub view_box: ViewBox,
    /// All paths that make up the SVG.
    pub paths: Vec<PathDescriptor>,
    /// The fully tessellated paths as [`Mesh`].
    pub mesh: Handle<Mesh>,
}

impl Svg {
    pub(crate) fn from_tree(tree: &usvg::Tree) -> Svg {
        let view_box = tree.view_box;
        let size = tree.size;
        let mut descriptors = Vec::new();

        for node in tree.root.descendants() {
            if let usvg::NodeKind::Path(ref path) = *node.borrow() {
                let t = path.transform;
                let abs_t = Transform::from_matrix(Mat4::from_cols(
                    [t.a.abs() as f32, t.b as f32, 0.0, 0.0].into(),
                    [t.c as f32, t.d.abs() as f32, 0.0, 0.0].into(),
                    [0.0, 0.0, 1.0, 0.0].into(),
                    [t.e as f32, t.f as f32, 0.0, 1.0].into(),
                ));

                if let Some(ref fill) = path.fill {
                    let color = match fill.paint {
                        usvg::Paint::Color(c) => Color::rgba_u8(
                            c.red,
                            c.green,
                            c.blue,
                            fill.opacity.to_u8(),
                        ),
                        _ => Color::default(),
                    };

                    descriptors.alloc().init(PathDescriptor {
                        segments: path.convert().collect(),
                        abs_transform: abs_t,
                        color,
                        draw_type: DrawType::Fill,
                    });
                }

                if let Some(ref stroke) = path.stroke {
                    let (color, draw_type) = stroke.convert();

                    descriptors.alloc().init(PathDescriptor {
                        segments: path.convert().collect(),
                        abs_transform: abs_t,
                        color,
                        draw_type,
                    });
                }
            }
        }

        Svg {
            name: Default::default(),
            size: Vec2::new(size.width() as f32, size.height() as f32),
            view_box: ViewBox {
                x: view_box.rect.x(),
                y: view_box.rect.y(),
                w: view_box.rect.width(),
                h: view_box.rect.height(),
            },
            paths: descriptors,
            mesh: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct PathDescriptor {
    pub segments: Vec<PathEvent>,
    pub abs_transform: Transform,
    pub color: Color,
    pub draw_type: DrawType,
}

#[derive(Debug)]
pub enum DrawType {
    Fill,
    Stroke(lyon_tessellation::StrokeOptions),
}

// Taken from https://github.com/nical/lyon/blob/74e6b137fea70d71d3b537babae22c6652f8843e/examples/wgpu_svg/src/main.rs
pub(crate) struct PathConvIter<'a> {
    iter: usvg::PathSegmentsIter<'a>,
    prev: Point,
    first: Point,
    needs_end: bool,
    deferred: Option<PathEvent>,
    scale: Transform2D<f32>,
}

impl<'l> Iterator for PathConvIter<'l> {
    type Item = PathEvent;

    fn next(&mut self) -> Option<Self::Item> {
        if self.deferred.is_some() {
            return self.deferred.take();
        }
        let mut return_event = None;
        let next = self.iter.next();
        match next {
            Some(usvg::PathSegment::MoveTo { x, y }) => {
                if self.needs_end {
                    let last = self.prev;
                    let first = self.first;
                    self.needs_end = false;
                    self.prev = (x, y).convert();
                    self.deferred = Some(PathEvent::Begin { at: self.prev });
                    self.first = self.prev;
                    return_event = Some(PathEvent::End {
                        last,
                        first,
                        close: false,
                    });
                } else {
                    self.first = (x, y).convert();
                    return_event = Some(PathEvent::Begin { at: self.first });
                }
            }
            Some(usvg::PathSegment::LineTo { x, y }) => {
                self.needs_end = true;
                let from = self.prev;
                self.prev = (x, y).convert();
                return_event = Some(PathEvent::Line {
                    from,
                    to: self.prev,
                });
            }
            Some(usvg::PathSegment::CurveTo {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            }) => {
                self.needs_end = true;
                let from = self.prev;
                self.prev = (x, y).convert();
                return_event = Some(PathEvent::Cubic {
                    from,
                    ctrl1: (x1, y1).convert(),
                    ctrl2: (x2, y2).convert(),
                    to: self.prev,
                });
            }
            Some(usvg::PathSegment::ClosePath) => {
                self.needs_end = false;
                self.prev = self.first;
                return_event = Some(PathEvent::End {
                    last: self.prev,
                    first: self.first,
                    close: true,
                });
            }
            None => {
                if self.needs_end {
                    self.needs_end = false;
                    let last = self.prev;
                    let first = self.first;
                    return_event = Some(PathEvent::End {
                        last,
                        first,
                        close: false,
                    });
                }
            }
        }

        return_event.map(|event| event.transformed(&self.scale))
    }
}

impl Convert<Point> for (&f64, &f64) {
    #[inline]
    fn convert(self) -> Point {
        Point::new((*self.0) as f32, (*self.1) as f32)
    }
}

impl Convert<Point> for (f64, f64) {
    #[inline]
    fn convert(self) -> Point {
        Point::new(self.0 as f32, self.1 as f32)
    }
}

impl<'a> Convert<PathConvIter<'a>> for &'a usvg::Path {
    fn convert(self) -> PathConvIter<'a> {
        return PathConvIter {
            iter: self.data.segments(),
            first: Point::new(0.0, 0.0),
            prev: Point::new(0.0, 0.0),
            deferred: None,
            needs_end: false,
            // For some reason the local transform of some paths has negative
            // scale values. Here we correct to positive values.
            scale: lyon_geom::Transform::scale(
                if self.transform.a < 0.0 { -1.0 } else { 1.0 },
                if self.transform.d < 0.0 { -1.0 } else { 1.0 },
            ),
        };
    }
}

impl Convert<(Color, DrawType)> for &usvg::Stroke {
    #[inline]
    fn convert(self) -> (Color, DrawType) {
        let color = match self.paint {
            usvg::Paint::Color(c) => {
                Color::rgba_u8(c.red, c.green, c.blue, self.opacity.to_u8())
            }
            usvg::Paint::LinearGradient(_)
            | usvg::Paint::RadialGradient(_)
            | usvg::Paint::Pattern(_) => Color::default(),
        };

        let linecap = match self.linecap {
            usvg::LineCap::Butt => lyon_tessellation::LineCap::Butt,
            usvg::LineCap::Square => lyon_tessellation::LineCap::Square,
            usvg::LineCap::Round => lyon_tessellation::LineCap::Round,
        };
        let linejoin = match self.linejoin {
            usvg::LineJoin::Miter => lyon_tessellation::LineJoin::Miter,
            usvg::LineJoin::Bevel => lyon_tessellation::LineJoin::Bevel,
            usvg::LineJoin::Round => lyon_tessellation::LineJoin::Round,
        };

        let opt = lyon_tessellation::StrokeOptions::tolerance(0.01)
            .with_line_width(self.width.get() as f32)
            .with_line_cap(linecap)
            .with_line_join(linejoin);

        (color, DrawType::Stroke(opt))
    }
}
