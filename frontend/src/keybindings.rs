use bridge::{FrontendMessage as Message, PanelAction};

use iced::widget::pane_grid::Direction;
use iced::{keyboard, mouse};

pub fn handle_hotkey(key_code: keyboard::KeyCode) -> Option<Message> {
    use keyboard::KeyCode;

    match key_code {
        // Bridge Messages
        KeyCode::Up | KeyCode::K => {
            Some(Message::PaneAction(PanelAction::FocusItem(Direction::Up)))
        }
        KeyCode::Down | KeyCode::J => {
            Some(Message::PaneAction(PanelAction::FocusItem(Direction::Down)))
        }
        KeyCode::Right | KeyCode::L => Some(Message::PaneAction(PanelAction::Enter)),
        KeyCode::Enter => Some(Message::PaneAction(PanelAction::Enter)),
        KeyCode::Left | KeyCode::H => Some(Message::PaneAction(PanelAction::Back)),
        KeyCode::R => Some(Message::PaneAction(PanelAction::Refresh)),
        KeyCode::W | KeyCode::Space => Some(Message::PaneAction(PanelAction::MarkEpisode)),
        KeyCode::PageDown => Some(Message::PaneAction(PanelAction::Plus(5))),
        KeyCode::PageUp => Some(Message::PaneAction(PanelAction::Plus(-5))),
        KeyCode::Home => Some(Message::PaneAction(PanelAction::Start)),
        KeyCode::End => Some(Message::PaneAction(PanelAction::End)),

        // Messages
        KeyCode::Q => Some(Message::CleanUp),
        _ => None,
    }
}

pub fn handle_mousewheel(delta: mouse::ScrollDelta) -> Option<Message> {
    if let mouse::ScrollDelta::Lines { x: _, y } = delta {
        if y > 0.0 {
            Some(Message::PaneAction(PanelAction::FocusItem(Direction::Up)))
        } else {
            Some(Message::PaneAction(PanelAction::FocusItem(Direction::Down)))
        }
    } else {
        None
    }
}

pub fn handle_mousebutton(button: mouse::Button) -> Option<Message> {
    match button {
        mouse::Button::Right | mouse::Button::Other(8) => {
            Some(Message::PaneAction(PanelAction::Back))
        }
        mouse::Button::Other(9) => Some(Message::PaneAction(PanelAction::Enter)),
        _ => None,
    }
}
