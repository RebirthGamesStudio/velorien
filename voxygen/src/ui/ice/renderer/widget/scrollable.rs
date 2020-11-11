use super::super::{super::Rotation, style, IcedRenderer, Primitive};
use common::util::srgba_to_linear;
use iced::{mouse, scrollable, Rectangle};
use style::scrollable::{Scroller, Track};

const SCROLLBAR_MIN_HEIGHT: u16 = 6;

impl scrollable::Renderer for IcedRenderer {
    type Style = style::scrollable::Style;

    // Interesting that this is here
    // I guess we can take advantage of this to keep a constant size despite
    // scaling?
    fn scrollbar(
        &self,
        bounds: Rectangle,
        content_bounds: Rectangle,
        offset: u32,
        scrollbar_width: u16,
        scrollbar_margin: u16,
        scroller_width: u16,
    ) -> Option<scrollable::Scrollbar> {
        if content_bounds.height > bounds.height {
            // Area containing both scrollbar and scroller
            let outer_width = (scrollbar_width.max(scroller_width) + 2 * scrollbar_margin) as f32;
            let outer_bounds = Rectangle {
                x: bounds.x + bounds.width - outer_width,
                width: outer_width,
                ..bounds
            };

            // Background scrollbar (i.e. the track)
            let scrollbar_bounds = Rectangle {
                x: bounds.x + bounds.width - outer_width / 2.0 - (scrollbar_width / 2) as f32,
                width: scrollbar_width as f32,
                ..bounds
            };

            // Interactive scroller
            let visible_fraction = bounds.height / content_bounds.height;
            let scroller_height =
                (bounds.height * visible_fraction).max((2 * SCROLLBAR_MIN_HEIGHT) as f32);
            let y_offset = offset as f32 * visible_fraction;

            let scroller_bounds = Rectangle {
                x: bounds.x + bounds.width - outer_width / 2.0 - (scrollbar_width / 2) as f32,
                y: scrollbar_bounds.y + y_offset,
                width: scroller_width as f32,
                height: scroller_height,
            };
            Some(scrollable::Scrollbar {
                outer_bounds,
                bounds: scrollbar_bounds,
                margin: scrollbar_margin,
                scroller: scrollable::Scroller {
                    bounds: scroller_bounds,
                },
            })
        } else {
            None
        }
    }

    fn draw(
        &mut self,
        state: &scrollable::State,
        bounds: Rectangle,
        _content_bounds: Rectangle,
        is_mouse_over: bool,
        is_mouse_over_scrollbar: bool,
        scrollbar: Option<scrollable::Scrollbar>,
        offset: u32,
        style_sheet: &Self::Style,
        (content, mouse_interaction): Self::Output,
    ) -> Self::Output {
        (
            if let Some(scrollbar) = scrollbar {
                let mut primitives = Vec::with_capacity(5);

                // Scrolled content
                primitives.push(Primitive::Clip {
                    bounds,
                    offset: (0, offset).into(),
                    content: Box::new(content),
                });

                let style = style_sheet;
                // Note: for future use if we vary style with the state of the scrollable
                //let style = if state.is_scroller_grabbed() {
                //    style_sheet.dragging()
                //} else if is_mouse_over_scrollbar {
                //    style_sheet.hovered()
                //} else {
                //    style_sheet.active();
                //};

                let is_scrollbar_visible = style.track.is_some();

                if is_mouse_over || state.is_scroller_grabbed() || is_scrollbar_visible {
                    let bounds = scrollbar.scroller.bounds;

                    match style.scroller {
                        Scroller::Color(color) => primitives.push(Primitive::Rectangle {
                            bounds,
                            linear_color: srgba_to_linear(color.map(|e| e as f32 / 255.0)),
                        }),
                        Scroller::Image { ends, mid, color } => {
                            // Calculate sizes of ends pieces based on image aspect ratio
                            let (img_w, img_h) = self.image_dims(ends);
                            let end_height = bounds.width * img_h as f32 / img_w as f32;

                            // Calcutate size of middle piece based on available space
                            // Note: Might want to scale into real pixels for parts of this
                            let (end_height, middle_height) =
                                if end_height * 2.0 + 1.0 <= bounds.height {
                                    (end_height, bounds.height - end_height * 2.0)
                                } else {
                                    // Take 1 logical pixel for the middle height
                                    let remaining_height = bounds.height - 1.0;
                                    (remaining_height / 2.0, 1.0)
                                };

                            // Top
                            primitives.push(Primitive::Image {
                                handle: (ends, Rotation::None),
                                bounds: Rectangle {
                                    height: end_height,
                                    ..bounds
                                },
                                color,
                                source_rect: None,
                            });
                            // Middle
                            primitives.push(Primitive::Image {
                                handle: (mid, Rotation::None),
                                bounds: Rectangle {
                                    y: bounds.y + end_height,
                                    height: middle_height,
                                    ..bounds
                                },
                                color,
                                source_rect: None,
                            });
                            // Bottom
                            primitives.push(Primitive::Image {
                                handle: (ends, Rotation::Cw180),
                                bounds: Rectangle {
                                    y: bounds.y + end_height + middle_height,
                                    height: end_height,
                                    ..bounds
                                },
                                color,
                                source_rect: None,
                            });
                        },
                    }
                }

                if let Some(track) = style.track {
                    primitives.push(match track {
                        Track::Color(color) => Primitive::Rectangle {
                            bounds: scrollbar.bounds,
                            linear_color: srgba_to_linear(color.map(|e| e as f32 / 255.0)),
                        },
                        Track::Image(handle, color) => Primitive::Image {
                            handle: (handle, Rotation::None),
                            bounds: scrollbar.bounds,
                            color,
                            source_rect: None,
                        },
                    });
                }

                Primitive::Group { primitives }
            } else {
                content
            },
            if is_mouse_over_scrollbar || state.is_scroller_grabbed() {
                mouse::Interaction::Idle
            } else {
                mouse_interaction
            },
        )
    }
}
