//! A container for capturing mouse events.

use iced_native::event::{self, Event};
use iced_native::layout;
use iced_native::mouse;
use iced_native::overlay;
use iced_native::renderer;
use iced_native::touch;
use iced_native::widget::{tree, Operation, Tree};
use iced_native::{Clipboard, Element, Layout, Length, Point, Rectangle, Shell, Widget};

/// A container intercepting mouse events.
pub fn mouse_area<'a, Message, Renderer>(
    widget: impl Into<Element<'a, Message, Renderer>>,
) -> MouseArea<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    MouseArea::new(widget)
}

/// Local state of the [`MouseArea`].
#[derive(Default)]
struct State {
    // TODO: Support on_mouse_enter and on_mouse_exit
    mouse_entered: bool,
}

/// Emit messages on mouse events.
#[allow(missing_debug_implementations)]
pub struct MouseArea<'a, Message, Renderer> {
    content: Element<'a, Message, Renderer>,
    on_area: Option<Message>,
    on_press: Option<Message>,
    on_release: Option<Message>,
    on_right_press: Option<Message>,
    on_right_release: Option<Message>,
    on_middle_press: Option<Message>,
    on_middle_release: Option<Message>,
}

impl<'a, Message, Renderer> MouseArea<'a, Message, Renderer> {
    /// Creates a [`MouseArea`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Renderer>>) -> Self {
        MouseArea {
            content: content.into(),
            on_area: None,
            on_press: None,
            on_release: None,
            on_right_press: None,
            on_right_release: None,
            on_middle_press: None,
            on_middle_release: None,
        }
    }
}

impl<'a, Message, Renderer> MouseArea<'a, Message, Renderer> {
    /// The message to emit when the mouse enters the widget area.
    #[must_use]
    pub fn on_area(mut self, message: Message) -> Self {
        self.on_area = Some(message);
        self
    }

    /// The message to emit on a left button press.
    #[must_use]
    pub fn on_press(mut self, message: Message) -> Self {
        self.on_press = Some(message);
        self
    }

    /// The message to emit on a left button release.
    #[must_use]
    pub fn on_release(mut self, message: Message) -> Self {
        self.on_release = Some(message);
        self
    }

    /// The message to emit on a right button press.
    #[must_use]
    pub fn on_right_press(mut self, message: Message) -> Self {
        self.on_right_press = Some(message);
        self
    }

    /// The message to emit on a right button release.
    #[must_use]
    pub fn on_right_release(mut self, message: Message) -> Self {
        self.on_right_release = Some(message);
        self
    }

    /// The message to emit on a middle button press.
    #[must_use]
    pub fn on_middle_press(mut self, message: Message) -> Self {
        self.on_middle_press = Some(message);
        self
    }

    /// The message to emit on a middle button release.
    #[must_use]
    pub fn on_middle_release(mut self, message: Message) -> Self {
        self.on_middle_release = Some(message);
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for MouseArea<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn width(&self) -> Length {
        self.content.as_widget().width()
    }

    fn height(&self) -> Length {
        self.content.as_widget().height()
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        self.content.as_widget().layout(renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        self.content
            .as_widget()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        if let event::Status::Captured = self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
        ) {
            return event::Status::Captured;
        }

        update(
            self,
            &event,
            layout,
            cursor_position,
            shell,
            tree.state.downcast_mut::<State>(),
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            renderer_style,
            layout,
            cursor_position,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(&mut tree.children[0], layout, renderer)
    }
}

impl<'a, Message, Renderer> From<MouseArea<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a + Clone,
    Renderer: 'a + iced_native::Renderer,
{
    fn from(area: MouseArea<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(area)
    }
}

/// Processes the given [`Event`] and updates the [`State`] of an [`MouseArea`]
/// accordingly.
fn update<Message: Clone, Renderer>(
    widget: &mut MouseArea<'_, Message, Renderer>,
    event: &Event,
    layout: Layout<'_>,
    cursor_position: Point,
    shell: &mut Shell<'_, Message>,
    state: &mut State,
) -> event::Status {
    if !layout.bounds().contains(cursor_position) {
        state.mouse_entered = false;
        return event::Status::Ignored;
    }

    if !state.mouse_entered {
        if let Some(message) = widget.on_area.as_ref() {
            if let Event::Mouse(mouse::Event::CursorMoved { position: _ }) = event {
                shell.publish(message.clone());
                state.mouse_entered = true;

                return event::Status::Captured;
            }
        }
    }

    if let Some(message) = widget.on_press.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) = event
        {
            shell.publish(message.clone());

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_release.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. }) = event
        {
            shell.publish(message.clone());

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_right_press.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) = event {
            shell.publish(message.clone());

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_right_release.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)) = event {
            shell.publish(message.clone());

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_middle_press.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) = event {
            shell.publish(message.clone());

            return event::Status::Captured;
        }
    }

    if let Some(message) = widget.on_middle_release.as_ref() {
        if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Middle)) = event {
            shell.publish(message.clone());

            return event::Status::Captured;
        }
    }

    event::Status::Ignored
}
