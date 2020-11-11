use super::super::{super::Rotation, style, IcedRenderer, Primitive};
use common::util::srgba_to_linear;
use iced::{container, Element, Layout, Point, Rectangle};
use style::container::Border;
use vek::Rgba;

// TODO: move to style
const BORDER_SIZE: u16 = 8;

impl container::Renderer for IcedRenderer {
    type Style = style::container::Style;

    fn draw<M>(
        &mut self,
        defaults: &Self::Defaults,
        bounds: Rectangle,
        cursor_position: Point,
        viewport: &Rectangle,
        style_sheet: &Self::Style,
        content: &Element<'_, M, Self>,
        content_layout: Layout<'_>,
    ) -> Self::Output {
        let (content, mouse_interaction) =
            content.draw(self, defaults, content_layout, cursor_position, viewport);

        let prim = match style_sheet {
            Self::Style::Image(handle, color) => {
                let background = Primitive::Image {
                    handle: (*handle, Rotation::None),
                    bounds,
                    color: *color,
                    source_rect: None,
                };

                Primitive::Group {
                    primitives: vec![background, content],
                }
            },
            Self::Style::Color(color, border) => {
                let linear_color = srgba_to_linear(color.map(|e| e as f32 / 255.0));

                let primitives = match border {
                    Border::None => {
                        let background = Primitive::Rectangle {
                            bounds,
                            linear_color,
                        };

                        vec![background, content]
                    },
                    Border::DoubleCornerless { inner, outer } => {
                        let border_size = f32::from(BORDER_SIZE)
                            .min(bounds.width / 4.0)
                            .min(bounds.height / 4.0);

                        let center = Primitive::Rectangle {
                            bounds: Rectangle {
                                x: bounds.x + border_size * 2.0,
                                y: bounds.y + border_size * 2.0,
                                width: bounds.width - border_size * 4.0,
                                height: bounds.height - border_size * 4.0,
                            },
                            linear_color,
                        };

                        let linear_color = srgba_to_linear(outer.map(|e| e as f32 / 255.0));
                        let top = Primitive::Rectangle {
                            bounds: Rectangle {
                                x: bounds.x + border_size,
                                y: bounds.y,
                                width: bounds.width - border_size * 2.0,
                                height: border_size,
                            },
                            linear_color,
                        };
                        let bottom = Primitive::Rectangle {
                            bounds: Rectangle {
                                x: bounds.x + border_size,
                                y: bounds.y + bounds.height - border_size,
                                width: bounds.width - border_size * 2.0,
                                height: border_size,
                            },
                            linear_color,
                        };
                        let left = Primitive::Rectangle {
                            bounds: Rectangle {
                                x: bounds.x,
                                y: bounds.y + border_size,
                                width: border_size,
                                height: bounds.height - border_size * 2.0,
                            },
                            linear_color,
                        };
                        let right = Primitive::Rectangle {
                            bounds: Rectangle {
                                x: bounds.x + bounds.width - border_size,
                                y: bounds.y + border_size,
                                width: border_size,
                                height: bounds.height - border_size * 2.0,
                            },
                            linear_color,
                        };

                        let linear_color = srgba_to_linear(inner.map(|e| e as f32 / 255.0));
                        let top_inner = Primitive::Rectangle {
                            bounds: Rectangle {
                                x: bounds.x + border_size,
                                y: bounds.y + border_size,
                                width: bounds.width - border_size * 2.0,
                                height: border_size,
                            },
                            linear_color,
                        };
                        let bottom_inner = Primitive::Rectangle {
                            bounds: Rectangle {
                                x: bounds.x + border_size,
                                y: bounds.y + bounds.height - border_size * 2.0,
                                width: bounds.width - border_size * 2.0,
                                height: border_size,
                            },
                            linear_color,
                        };
                        let left_inner = Primitive::Rectangle {
                            bounds: Rectangle {
                                x: bounds.x + border_size,
                                y: bounds.y + border_size * 2.0,
                                width: border_size,
                                height: bounds.height - border_size * 4.0,
                            },
                            linear_color,
                        };
                        let right_inner = Primitive::Rectangle {
                            bounds: Rectangle {
                                x: bounds.x + bounds.width - border_size * 2.0,
                                y: bounds.y + border_size * 2.0,
                                width: border_size,
                                height: bounds.height - border_size * 4.0,
                            },
                            linear_color,
                        };

                        vec![
                            center,
                            top,
                            bottom,
                            left,
                            right,
                            top_inner,
                            bottom_inner,
                            left_inner,
                            right_inner,
                            content,
                        ]
                    },
                    Border::Image { corner, edge } => {
                        let border_size = f32::from(BORDER_SIZE)
                            .min(bounds.width / 4.0)
                            .min(bounds.height / 4.0);

                        let center = Primitive::Rectangle {
                            bounds: Rectangle {
                                x: bounds.x + border_size,
                                y: bounds.y + border_size,
                                width: bounds.width - border_size * 2.0,
                                height: bounds.height - border_size * 2.0,
                            },
                            linear_color,
                        };

                        let color = Rgba::white();

                        let tl_corner = Primitive::Image {
                            handle: (*corner, Rotation::None),
                            bounds: Rectangle {
                                x: bounds.x,
                                y: bounds.y,
                                width: border_size,
                                height: border_size,
                            },
                            color,
                            source_rect: None,
                        };

                        let tr_corner = Primitive::Image {
                            handle: (*corner, Rotation::Cw90),
                            bounds: Rectangle {
                                x: bounds.x + bounds.width - border_size,
                                y: bounds.y,
                                width: border_size,
                                height: border_size,
                            },
                            color,
                            source_rect: None,
                        };

                        let bl_corner = Primitive::Image {
                            handle: (*corner, Rotation::Cw270),
                            bounds: Rectangle {
                                x: bounds.x,
                                y: bounds.y + bounds.height - border_size,
                                width: border_size,
                                height: border_size,
                            },
                            color,
                            source_rect: None,
                        };

                        let br_corner = Primitive::Image {
                            handle: (*corner, Rotation::Cw180),
                            bounds: Rectangle {
                                x: bounds.x + bounds.width - border_size,
                                y: bounds.y + bounds.height - border_size,
                                width: border_size,
                                height: border_size,
                            },
                            color,
                            source_rect: None,
                        };

                        let top_edge = Primitive::Image {
                            handle: (*edge, Rotation::None),
                            bounds: Rectangle {
                                x: bounds.x + border_size,
                                y: bounds.y,
                                width: bounds.width - 2.0 * border_size,
                                height: border_size,
                            },
                            color,
                            source_rect: None,
                        };

                        let bottom_edge = Primitive::Image {
                            handle: (*edge, Rotation::Cw180),
                            bounds: Rectangle {
                                x: bounds.x + border_size,
                                y: bounds.y + bounds.height - border_size,
                                width: bounds.width - 2.0 * border_size,
                                height: border_size,
                            },
                            color,
                            source_rect: None,
                        };

                        let left_edge = Primitive::Image {
                            handle: (*edge, Rotation::Cw270),
                            bounds: Rectangle {
                                x: bounds.x,
                                y: bounds.y + border_size,
                                width: border_size,
                                height: bounds.height - 2.0 * border_size,
                            },
                            color,
                            source_rect: None,
                        };

                        let right_edge = Primitive::Image {
                            handle: (*edge, Rotation::Cw90),
                            bounds: Rectangle {
                                x: bounds.x + bounds.width - border_size,
                                y: bounds.y + border_size,
                                width: border_size,
                                height: bounds.height - 2.0 * border_size,
                            },
                            color,
                            source_rect: None,
                        };

                        // Is this worth it as opposed to using a giant image? (Probably)
                        vec![
                            center,
                            tl_corner,
                            tr_corner,
                            bl_corner,
                            br_corner,
                            top_edge,
                            bottom_edge,
                            left_edge,
                            right_edge,
                            content,
                        ]
                    },
                };

                Primitive::Group { primitives }
            },
            Self::Style::None => content,
        };

        (prim, mouse_interaction)
    }
}
