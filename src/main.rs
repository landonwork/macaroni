use std::io;
use tempfile::TempDir;

use iced::widget::{
    button, column, container, horizontal_space, row, text,
    text_editor
};
use iced::{
    Theme, Element, Length, Application, Settings, Command,
    Subscription, Event, Font
};
use iced::{executor, event, window};
use iced_highlighter::Highlighter;


fn main() -> iced::Result {
    Macaroni::run(
        Settings {
            default_font: Font::MONOSPACE,
            ..Settings::default()
        }
    )
}

struct TempDirs {
    attribute: TempDir,
    declarative: TempDir,
    derive: TempDir,
    function: TempDir,
}

impl TempDirs {
    fn new() -> Self {
        Self {
            attribute: TempDir::new().unwrap(),
            declarative: TempDir::new().unwrap(),
            derive: TempDir::new().unwrap(),
            function: TempDir::new().unwrap(),
        }
    }
}

#[derive(Debug)]
pub enum MacroType {
    Attribute,
    Declarative,
    Derive,
    Function,
}

pub struct Macaroni {
    temp_dirs: Option<TempDirs>,
    src_code: text_editor::Content,
    test_code: text_editor::Content,
    expansion: String,
    macro_type: Option<MacroType>,
    error: Option<Error>
}

#[derive(Clone, Debug)]
pub enum Message {
    CloseRequested,
    EditSource(text_editor::Action),
    EditTest(text_editor::Action),
    ExpandRequested,
    Expanded(Result<String>),
    New,
}

impl Application for Macaroni {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                temp_dirs: Some(TempDirs::new()),
                src_code: text_editor::Content::new(),
                test_code: text_editor::Content::new(),
                expansion: String::new(),
                macro_type: None,
                error: None
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        String::from("Macaroni - Rust macros are easy!")
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _| {
            if let Event::Window(window::Event::CloseRequested) = event {
                Some(Message::CloseRequested)
            } else {
                None
            }
        })
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::CloseRequested => {
                self.temp_dirs = None;
                window::close()
            }
            Message::EditSource(action) => {
                self.src_code.perform(action);
                self.error = None;
                Command::none()
            }
            Message::EditTest(action) => {
                self.src_code.perform(action);
                self.error = None;
                Command::none()
            }
            Message::ExpandRequested => {
                let dir = match self.macro_type {
                    Some(MacroType::Declarative) => self.temp_dirs.as_ref().unwrap().declarative.path().to_str().unwrap().to_owned(),
                    Some(MacroType::Attribute) => self.temp_dirs.as_ref().unwrap().attribute.path().to_str().unwrap().to_owned(),
                    Some(MacroType::Derive) => self.temp_dirs.as_ref().unwrap().derive.path().to_str().unwrap().to_owned(),
                    Some(MacroType::Function) => self.temp_dirs.as_ref().unwrap().function.path().to_str().unwrap().to_owned(),
                    None => unreachable!()
                };
                Command::perform(
                    actions::expand(dir, self.src_code.text(), self.test_code.text()),
                    Message::Expanded
                )
            }
            Message::Expanded(Ok(text)) => {
                self.expansion = text;
                Command::none()
            }
            Message::Expanded(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }
            Message::New => {
                // TODO
                self.src_code = text_editor::Content::new();
                self.test_code = text_editor::Content::new();
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let controls = row![
            button("New").on_press(Message::New),
            // button("Open").on_press(Message::Open),
            // button("Save").on_press(Message::Save),
            // button("Expand").on_press(Message::ExpandRequested)
        ]
        .spacing(10);
        let src = elements::src_input(self);
        let test = elements::test_input(self);
        // let expansion = elements::expansion();
        let status_bar = elements::status_bar(self);

        container(column![controls, column![src, test].spacing(20), status_bar].spacing(10)).padding(10).into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    CargoFailed(String),
    IOFailed(io::ErrorKind)
}

pub mod actions {
    use super::*;

    pub async fn expand(dir: String, src_code: String, test_code: String) -> Result<String> {
        tokio::fs::write(format!("{}/src/lib.rs", dir), src_code).await
            .map_err(|e| Error::IOFailed(e.kind()))?;
        tokio::fs::write(format!("{}/tests/test.rs", dir), test_code).await
            .map_err(|e| Error::IOFailed(e.kind()))?;
        match tokio::process::Command::new("cargo")
            .current_dir(dir)
            .arg("expand")
            .arg("tests/test.rs")
            .output().await
        {
            Ok(output) => {
                if output.stdout.is_empty() {
                    Err(Error::CargoFailed(std::str::from_utf8(&output.stderr).unwrap().to_owned()))
                } else {
                    Ok(std::str::from_utf8(&output.stdout).unwrap().to_owned())
                }
            }
            Err(error) => {
                Err(Error::IOFailed(error.kind()))
            }
        }
    }
    // async fn pick_file() -> Result<(PathBuf, Arc<String>), Error> {
    //     let handle = rfd::AsyncFileDialog::new()
    //         .set_title("Choose a text file...")
    //         .pick_file().await
    //         .ok_or(Error::DialogClosed)?;
    //     load_file(handle.path().to_owned()).await
    // }

    // async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {
    //     tokio::fs::read_to_string(&path).await
    //         .map(|x| (path, Arc::new(x)))
    //         .map_err(|error| Error::IOFailed(error.kind()))
    // }

    // async fn save_file(path: Option<PathBuf>, text: String) -> Result<PathBuf, Error> {
    //     let path = if let Some(path) = path {
    //         path
    //     } else {
    //         rfd::AsyncFileDialog::new()
    //             .set_title("Choose a file name...")
    //             .save_file().await
    //             .ok_or(Error::DialogClosed)
    //             .map(|handle| handle.path().to_owned())?
    //     };

    //     tokio::fs::write(&path, text).await
    //         .map_err(|error| Error::IOFailed(error.kind()))?;

    //     Ok(path)
    // }
}

pub mod elements {
    use super::*;
    pub fn status_bar(slf: &Macaroni) -> Element<'_, Message> {
        let status = if let Some(Error::IOFailed(error)) = slf.error.as_ref() {
            text(error.to_string())
        } else {
            match slf.macro_type {
                Some(ref t) => text(format!("{:?}", t)),
                None => text(String::new())
            }
        };
        row![status, horizontal_space(Length::Fill)].into()
    }

    pub fn src_input<'a>(slf: &'a Macaroni) -> Element<'a, Message> {
        text_editor(&slf.src_code)
        .on_action(Message::EditSource)
        .highlight::<Highlighter>(
            iced_highlighter::Settings {
                theme: iced_highlighter::Theme::SolarizedDark,
                extension: "rs".to_string()
            },
            |highlight, _theme| { highlight.to_format() }
        )
        .into()
    }

    pub fn test_input<'a>(slf: &'a Macaroni) -> Element<'a, Message> {
        text_editor(&slf.test_code)
        .on_action(Message::EditTest)
        .highlight::<Highlighter>(
            iced_highlighter::Settings {
                theme: iced_highlighter::Theme::SolarizedDark,
                extension: "rs".to_string()
            },
            |highlight, _theme| { highlight.to_format() }
        )
        .into()
    }
}

// fn action<'a>(
//     content: Element<'a, Message>,
//     label: &str,
//     on_press: Message,
// ) -> Element<'a, Message> {
//     tooltip(
//         button(container(content).width(3).center_x()) .on_press(on_press),
//         label,
//         tooltip::Position::FollowCursor
//     )
//     .padding(10)
//     .style(theme::Container::Box)
//     .into()
// }
