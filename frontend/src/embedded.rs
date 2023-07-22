pub static EMPTY_SVG: &[u8] = include_bytes!("../../res/svgs/empty.svg");
pub static RELOAD_SVG: &[u8] = include_bytes!("../../res/svgs/reload.svg");
pub static CHECKMARK_SVG: &[u8] = include_bytes!("../../res/svgs/checkmark.svg");
pub static CHECKMARK_P_SVG: &[u8] = include_bytes!("../../res/svgs/checkmark_previous.svg");
pub static FOLDER_SVG: &[u8] = include_bytes!("../../res/svgs/folder.svg");

pub static NO_TUMBNAIL: &[u8] = include_bytes!("../../res/no_thumbnail.jpg");
pub static YAMA_ICON: &[u8] = include_bytes!("../../res/yama_logo.ico");
pub static YAMA_PNG: &[u8] = include_bytes!("../../res/yama.png");

pub static REGULAR_FONT_BYTES: &[u8] = include_bytes!("../../res/fonts/KumbhSans-Regular.ttf");

pub static BOLD_FONT: iced::Font = iced::Font::External {
    name: "Kumbh Sans Bold",
    bytes: include_bytes!("../../res/fonts/KumbhSans-Bold.ttf"),
};
