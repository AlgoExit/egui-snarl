use egui::{Color32, Painter, Rect, Shape, Stroke, Style, Vec2, epaint::PathShape, pos2, vec2};

use crate::{InPinId, OutPinId};

use super::{SnarlStyle, WireAxis, WireStyle};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnyPin {
    Out(OutPinId),
    In(InPinId),
}

/// In the current context, these are the I/O pins of the 'source' node that the newly
/// created node's I/O pins will connect to.
#[derive(Debug)]
pub enum AnyPins<'a> {
    /// Output pins.
    Out(&'a [OutPinId]),
    /// Input pins
    In(&'a [InPinId]),
}

/// Contains information about a pin's wire.
/// Used to draw the wire.
/// When two pins are connected, the wire is drawn between them,
/// using merged `PinWireInfo` from both pins.
pub struct PinWireInfo {
    /// Desired color of the wire.
    pub color: Color32,

    /// Desired style of the wire.
    /// Zoomed with current scale.
    pub style: WireStyle,

    /// Desired axis of the wire, if the pin asks for one.
    pub axis: Option<WireAxis>,
}

/// Where a pin sits on its node, for implementations that place pins themselves.
///
/// The default placement puts inputs on the left edge and outputs on the right,
/// which is all `x`, `y0` and `y1` can express. Laying pins out along the top and
/// bottom edges instead — a top-to-bottom flow — needs the node's horizontal
/// extent and the pin's ordinal, neither of which was reachable from `pin_rect`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PinLayout {
    /// Frame rect of the whole node this pin belongs to.
    pub node_rect: Rect,

    /// Index of this pin among the node's inputs, or among its outputs.
    pub index: usize,

    /// Number of the node's inputs, or of its outputs.
    pub count: usize,

    /// `true` for an input pin, `false` for an output pin.
    pub is_input: bool,
}

impl PinLayout {
    /// Evenly spaced offset of this pin along an edge, in `0.0..=1.0`.
    ///
    /// `count + 1` gaps rather than `count`, so the pins sit inside the edge
    /// instead of touching its corners, and a single pin lands in the middle.
    #[must_use]
    pub fn fraction_along_edge(&self) -> f32 {
        #[allow(clippy::cast_precision_loss)]
        {
            (self.index as f32 + 1.0) / (self.count as f32 + 1.0)
        }
    }
}

/// Uses `Painter` to draw a pin.
pub trait SnarlPin {
    /// Calculates pin Rect from the given parameters.
    ///
    /// `x` is the edge the default placement uses, `y0..=y1` is the pin's row.
    /// `layout` additionally describes the node and the pin's ordinal, so an
    /// implementation may place the pin anywhere on the node instead.
    fn pin_rect(&self, layout: PinLayout, x: f32, y0: f32, y1: f32, size: f32) -> Rect {
        let _ = layout;
        // Center vertically by default.
        let y = (y0 + y1) * 0.5;
        let pin_pos = pos2(x, y);
        Rect::from_center_size(pin_pos, vec2(size, size))
    }

    /// Draws the pin.
    ///
    /// `rect` is the interaction rectangle of the pin.
    /// Pin should fit in it.
    /// `painter` is used to add pin's shapes to the UI.
    ///
    /// Returns the color
    #[must_use]
    fn draw(
        self,
        snarl_style: &SnarlStyle,
        style: &Style,
        rect: Rect,
        painter: &Painter,
    ) -> PinWireInfo;
}

/// Shape of a pin.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "egui-probe", derive(egui_probe::EguiProbe))]
pub enum PinShape {
    /// Circle shape.
    #[default]
    Circle,

    /// Triangle shape.
    Triangle,

    /// Square shape.
    Square,

    /// Star shape.
    Star,
}

/// Information about a pin returned by `SnarlViewer::show_input` and `SnarlViewer::show_output`.
///
/// All fields are optional.
/// If a field is `None`, the default value is used derived from the graph style.
#[derive(Default)]
pub struct PinInfo {
    /// Shape of the pin.
    pub shape: Option<PinShape>,

    /// Fill color of the pin.
    pub fill: Option<Color32>,

    /// Outline stroke of the pin.
    pub stroke: Option<Stroke>,

    /// Color of the wire connected to the pin.
    /// If `None`, the pin's fill color is used.
    pub wire_color: Option<Color32>,

    /// Style of the wire connected to the pin.
    pub wire_style: Option<WireStyle>,

    /// Axis the wire connected to the pin is routed along.
    ///
    /// Overrides [`SnarlStyle::wire_axis`] for this pin. Needed when a node
    /// carries pins of more than one class — an execution chain leaving the
    /// bottom edge and data entering from the side — because then the routing
    /// has to match the edge the wire actually ends on, and one axis for the
    /// whole editor can no longer be right for every wire.
    pub wire_axis: Option<WireAxis>,

    /// Custom vertical position of a pin
    pub position: Option<f32>,
}

impl PinInfo {
    /// Sets the axis the pin's wire is routed along.
    #[must_use]
    pub const fn with_wire_axis(mut self, axis: WireAxis) -> Self {
        self.wire_axis = Some(axis);
        self
    }

    /// Sets the shape of the pin.
    #[must_use]
    pub const fn with_shape(mut self, shape: PinShape) -> Self {
        self.shape = Some(shape);
        self
    }

    /// Sets the fill color of the pin.
    #[must_use]
    pub const fn with_fill(mut self, fill: Color32) -> Self {
        self.fill = Some(fill);
        self
    }

    /// Sets the outline stroke of the pin.
    #[must_use]
    pub const fn with_stroke(mut self, stroke: Stroke) -> Self {
        self.stroke = Some(stroke);
        self
    }

    /// Sets the style of the wire connected to the pin.
    #[must_use]
    pub const fn with_wire_style(mut self, wire_style: WireStyle) -> Self {
        self.wire_style = Some(wire_style);
        self
    }

    /// Sets the color of the wire connected to the pin.
    #[must_use]
    pub const fn with_wire_color(mut self, wire_color: Color32) -> Self {
        self.wire_color = Some(wire_color);
        self
    }

    /// Creates a circle pin.
    #[must_use]
    pub fn circle() -> Self {
        PinInfo {
            shape: Some(PinShape::Circle),
            ..Default::default()
        }
    }

    /// Creates a triangle pin.
    #[must_use]
    pub fn triangle() -> Self {
        PinInfo {
            shape: Some(PinShape::Triangle),
            ..Default::default()
        }
    }

    /// Creates a square pin.
    #[must_use]
    pub fn square() -> Self {
        PinInfo {
            shape: Some(PinShape::Square),
            ..Default::default()
        }
    }

    /// Creates a star pin.
    #[must_use]
    pub fn star() -> Self {
        PinInfo {
            shape: Some(PinShape::Star),
            ..Default::default()
        }
    }

    /// Returns the shape of the pin.
    #[must_use]
    pub fn get_shape(&self, snarl_style: &SnarlStyle) -> PinShape {
        self.shape.unwrap_or_else(|| snarl_style.get_pin_shape())
    }

    /// Returns fill color of the pin.
    #[must_use]
    pub fn get_fill(&self, snarl_style: &SnarlStyle, style: &Style) -> Color32 {
        self.fill.unwrap_or_else(|| snarl_style.get_pin_fill(style))
    }

    /// Returns outline stroke of the pin.
    #[must_use]
    pub fn get_stroke(&self, snarl_style: &SnarlStyle, style: &Style) -> Stroke {
        self.stroke
            .unwrap_or_else(|| snarl_style.get_pin_stroke(style))
    }

    /// Draws the pin and returns color.
    ///
    /// Wires are drawn with returned color by default.
    #[must_use]
    pub fn draw(
        &self,
        snarl_style: &SnarlStyle,
        style: &Style,
        rect: Rect,
        painter: &Painter,
    ) -> PinWireInfo {
        let shape = self.get_shape(snarl_style);
        let fill = self.get_fill(snarl_style, style);
        let stroke = self.get_stroke(snarl_style, style);
        draw_pin(painter, shape, fill, stroke, rect);

        PinWireInfo {
            color: self.wire_color.unwrap_or(fill),
            style: self
                .wire_style
                .unwrap_or_else(|| snarl_style.get_wire_style()),
            axis: self.wire_axis,
        }
    }
}

impl SnarlPin for PinInfo {
    fn draw(
        self,
        snarl_style: &SnarlStyle,
        style: &Style,
        rect: Rect,
        painter: &Painter,
    ) -> PinWireInfo {
        Self::draw(&self, snarl_style, style, rect, painter)
    }
}

pub fn draw_pin(painter: &Painter, shape: PinShape, fill: Color32, stroke: Stroke, rect: Rect) {
    let center = rect.center();
    let size = f32::min(rect.width(), rect.height());

    match shape {
        PinShape::Circle => {
            painter.circle(center, size / 2.0, fill, stroke);
        }
        PinShape::Triangle => {
            const A: Vec2 = vec2(-0.649_519, 0.4875);
            const B: Vec2 = vec2(0.649_519, 0.4875);
            const C: Vec2 = vec2(0.0, -0.6375);

            let points = vec![center + A * size, center + B * size, center + C * size];

            painter.add(Shape::Path(PathShape {
                points,
                closed: true,
                fill,
                stroke: stroke.into(),
            }));
        }
        PinShape::Square => {
            let points = vec![
                center + vec2(-0.5, -0.5) * size,
                center + vec2(0.5, -0.5) * size,
                center + vec2(0.5, 0.5) * size,
                center + vec2(-0.5, 0.5) * size,
            ];

            painter.add(Shape::Path(PathShape {
                points,
                closed: true,
                fill,
                stroke: stroke.into(),
            }));
        }

        PinShape::Star => {
            let points = vec![
                center + size * 0.700_000 * vec2(0.0, -1.0),
                center + size * 0.267_376 * vec2(-0.587_785, -0.809_017),
                center + size * 0.700_000 * vec2(-0.951_057, -0.309_017),
                center + size * 0.267_376 * vec2(-0.951_057, 0.309_017),
                center + size * 0.700_000 * vec2(-0.587_785, 0.809_017),
                center + size * 0.267_376 * vec2(0.0, 1.0),
                center + size * 0.700_000 * vec2(0.587_785, 0.809_017),
                center + size * 0.267_376 * vec2(0.951_057, 0.309_017),
                center + size * 0.700_000 * vec2(0.951_057, -0.309_017),
                center + size * 0.267_376 * vec2(0.587_785, -0.809_017),
            ];

            painter.add(Shape::Path(PathShape {
                points,
                closed: true,
                fill,
                stroke: stroke.into(),
            }));
        }
    }
}
