use crate::ui::ice as ui;
use iced::{button::State, Button, Element, Length};
use ui::{
    widget::{AspectRatioContainer, FillText},
    ButtonStyle,
};

pub fn neat_button<M: Clone + 'static>(
    state: &mut State,
    label: impl Into<String>,
    fill_fraction: f32,
    button_style: ButtonStyle,
    message: Option<M>,
) -> Element<M, ui::IcedRenderer> {
    let button = Button::new(state, FillText::new(label).fill_fraction(fill_fraction))
        .height(Length::Fill)
        .width(Length::Fill)
        .style(button_style);

    let button = match message {
        Some(message) => button.on_press(message),
        None => button,
    };

    let container = AspectRatioContainer::new(button);
    let container = match button_style.active().0 {
        Some(img) => container.ratio_of_image(img),
        None => container,
    };

    container.into()
}
