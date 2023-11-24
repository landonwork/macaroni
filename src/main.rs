use std::{io, fs, path::{Path, PathBuf}};
use tempfile::TempDir;

use iced::widget::{
    button, column, container, horizontal_space, row, text,
    text_editor, Column
};
use iced::{
    Theme, Element, Length, Application, Settings, Command,
    Subscription, Event, Font, Alignment
};
use iced::{executor, event, subscription, window};
use iced_highlighter::Highlighter;


fn main() -> iced::Result {
    Macaroni::run(
        Settings {
            default_font: Font::MONOSPACE,
            ..Settings::default()
        }
    )
}

struct Project {
    toml: &'static [u8],
    lib_rs: &'static [u8],
    test_rs: &'static [u8],
}

impl Project {
    fn write(&self, path: &Path) {
        fs::write(path.join("Cargo.toml"), self.toml).unwrap();
        fs::create_dir(path.join("src")).unwrap();
        fs::write(path.join("src/lib.rs"), self.lib_rs).unwrap();
        fs::create_dir(path.join("tests")).unwrap();
        fs::write(path.join("tests/test.rs"), self.test_rs).unwrap();
    }

    fn make_new(&self) -> (text_editor::Content, text_editor::Content, text_editor::Content) {
        let src_code = text_editor::Content::with_text(unsafe { std::str::from_utf8_unchecked(FRESH_ATTRIBUTE.lib_rs) });
        let test_code = text_editor::Content::with_text(unsafe { std::str::from_utf8_unchecked(FRESH_ATTRIBUTE.test_rs) });
        let expansion = text_editor::Content::new();

        (src_code, test_code, expansion)
    }

    fn read(path: &Path) -> (text_editor::Content, text_editor::Content, text_editor::Content) {
        let src_code = text_editor::Content::with_text(
            &fs::read_to_string(path.join("src/lib.rs")).unwrap()
        );
        let test_code = text_editor::Content::with_text(
            &fs::read_to_string(path.join("tests/test.rs")).unwrap()
        );
        let expansion = text_editor::Content::new();

        (src_code, test_code, expansion)
    }
}

const FRESH_ATTRIBUTE: Project = Project {
    toml: include_bytes!("../assets/attribute/Cargo.toml"),
    lib_rs: include_bytes!("../assets/attribute/src/lib.rs"),
    test_rs: include_bytes!("../assets/attribute/tests/test.rs"),
};
// const FRESH_DECLARATIVE: Project = Project {
//     toml: include_bytes!("../assets/declarative/Cargo.toml"),
//     lib_rs: include_bytes!("../assets/declarative/src/lib.rs"),
//     test_rs: include_bytes!("../assets/declarative/tests/test.rs"),
// };
// const FRESH_DERIVE: Project = Project {
//     toml: include_bytes!("../assets/derive/Cargo.toml"),
//     lib_rs: include_bytes!("../assets/derive/src/lib.rs"),
//     test_rs: include_bytes!("../assets/derive/tests/test.rs"),
// };
// const FRESH_FUNCTION: Project = Project {
//     toml: include_bytes!("../assets/function/Cargo.toml"),
//     lib_rs: include_bytes!("../assets/function/src/lib.rs"),
//     test_rs: include_bytes!("../assets/function/tests/test.rs"),
// };

struct TempDirs {
    attribute: TempDir,
    declarative: TempDir,
    derive: TempDir,
    function: TempDir,
}

impl TempDirs {
    fn new() -> Self {
        let slf = Self {
            attribute: TempDir::with_prefix("mac-attribute").unwrap(),
            declarative: TempDir::with_prefix("mac-declarative").unwrap(),
            derive: TempDir::with_prefix("mac-derive").unwrap(),
            function: TempDir::with_prefix("mac-function").unwrap(),
        };

        let path = slf.attribute.path();
        FRESH_ATTRIBUTE.write(path);

        slf
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
    expansion: text_editor::Content,
    macro_type: Option<MacroType>,
    error: Option<Error>
}

#[derive(Clone, Debug)]
pub enum Message {
    CloseRequested,
    EditExpansion(text_editor::Action),
    EditSource(text_editor::Action),
    EditTest(text_editor::Action),
    ExpandRequested,
    Expanded(Result<String>),
    Ignore,
    Navigate(Option<MacroType>),
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
                expansion: text_editor::Content::new(),
                macro_type: None,
                error: None
            },
            window::maximize(true)
        )
    }

    fn title(&self) -> String {
        String::from("Macaroni - Rust macros are easy!")
    }

    fn subscription(&self) -> Subscription<Message> {
        let close_req = event::listen_with(|event, _|
            if let Event::Window(window::Event::CloseRequested) = event {
                Some(Message::CloseRequested)
            } else {
                None
            }
        );
        let ctrl_c = subscription::run(
            move || {
                let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
                let _ = ctrlc::set_handler(move || { let _ = sender.blocking_send(()); });
                futures::stream::once(async move {
                    receiver.recv().await;
                    Message::CloseRequested
                })
            }
        );
        subscription::Subscription::batch([close_req, ctrl_c])
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::CloseRequested => {
                self.temp_dirs = None;
                window::close()
            }
            Message::EditExpansion(action) => {
                self.expansion.perform(action);
                Command::none()
            }
            Message::EditSource(action) => {
                self.src_code.perform(action);
                self.error = None;
                Command::none()
            }
            Message::EditTest(action) => {
                self.test_code.perform(action);
                self.error = None;
                Command::none()
            }
            Message::ExpandRequested => {
                let dir = match self.macro_type {
                    Some(MacroType::Declarative) => self.temp_dirs.as_ref().unwrap().declarative.path().to_owned(),
                    Some(MacroType::Attribute) => self.temp_dirs.as_ref().unwrap().attribute.path().to_owned(),
                    Some(MacroType::Derive) => self.temp_dirs.as_ref().unwrap().derive.path().to_owned(),
                    Some(MacroType::Function) => self.temp_dirs.as_ref().unwrap().function.path().to_owned(),
                    None => unreachable!()
                };
                Command::perform(
                    actions::expand(dir, self.src_code.text(), self.test_code.text()),
                    Message::Expanded
                )
            }
            Message::Expanded(Ok(text)) => {
                self.expansion = text_editor::Content::with_text(&text);
                Command::none()
            }
            Message::Expanded(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }
            Message::Ignore => Command::none(),
            Message::Navigate(macro_type) => {
                if macro_type != self.macro_type {
                    self.macro_type = macro_type;
                    if let Some(ref macro_type) = self.macro_type {
                        let dir = match macro_type {
                            MacroType::Attribute => self.temp_dirs.as_ref().unwrap().attribute.path(),
                            _ => todo!()
                        };
                        (self.src_code, self.test_code, self.expansion) = Project::read(dir);
                    }
                }
                Command::none()
            }
            Message::New => {
                match self.macro_type {
                    Some(MacroType::Attribute) => {
                        (self.src_code, self.test_code, self.expansion) = FRESH_ATTRIBUTE.make_new();
                    }
                    _ => {}
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let title_bar = row![horizontal_space(Length::Fill), text("Macaroni"), horizontal_space(Length::Fill)].align_items(Alignment::Center);
        let controls = if self.macro_type.is_some() {
            row![
                button("New").on_press(Message::New),
                horizontal_space(Length::Fill),
                button("README").on_press(Message::Navigate(None)),
                button("Attribute").on_press(Message::Navigate(Some(MacroType::Attribute))),
                button("Declarative").on_press(Message::Navigate(Some(MacroType::Declarative))),
                button("Derive").on_press(Message::Navigate(Some(MacroType::Derive))),
                button("Function").on_press(Message::Navigate(Some(MacroType::Function))),
                horizontal_space(Length::Fill),
                button("Expand").on_press(Message::ExpandRequested)
            ]
        } else {
            row![
                horizontal_space(Length::Fill),
                button("README").on_press(Message::Navigate(None)),
                button("Attribute").on_press(Message::Navigate(Some(MacroType::Attribute))),
                button("Declarative").on_press(Message::Navigate(Some(MacroType::Declarative))),
                button("Derive").on_press(Message::Navigate(Some(MacroType::Derive))),
                button("Function").on_press(Message::Navigate(Some(MacroType::Function))),
                horizontal_space(Length::Fill),
            ]
        }
        .spacing(10);

        let src = elements::editor(&self.src_code, Message::EditSource);
        let test = elements::editor(&self.test_code, Message::EditTest);
        let ignore_edits = |action| match action {
            text_editor::Action::Edit(_) => Message::Ignore,
            action => Message::EditExpansion(action),
        };
        let expansion = elements::editor(&self.expansion, ignore_edits);

        let column1 = Column::new()
            .width(Length::Fill)
            .spacing(20)
            .push(src)
            .push(test);
        let column2 = Column::new()
            .width(Length::Fill)
            .spacing(20)
            .push(expansion);

        let status_bar = elements::status_bar(self);

        container(
            column![
                title_bar,
                controls,
                row![column1, column2].spacing(20),
                status_bar
            ]
            .spacing(10)
        )
        .padding(10)
        .into()
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

    pub async fn expand(dir: PathBuf, src_code: String, test_code: String) -> Result<String> {
        tokio::fs::write(dir.join("src/lib.rs"), src_code).await
            .map_err(|e| Error::IOFailed(e.kind()))?;
        tokio::fs::write(dir.join("tests/test.rs"), test_code).await
            .map_err(|e| Error::IOFailed(e.kind()))?;
        match tokio::process::Command::new("cargo")
            .current_dir(dir)
            .arg("expand")
            .arg("--test")
            .arg("test")
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
}

pub mod elements {
    use super::*;
    pub fn status_bar(slf: &Macaroni) -> Element<'_, Message> {
        let status = if let Some(Error::IOFailed(error)) = slf.error.as_ref() {
            text(error.to_string())
        } else {
            match slf.macro_type {
                Some(ref t) => text(format!("{:?}", t)),
                None => text("Home")
            }
        };
        row![status, horizontal_space(Length::Fill)].into()
    }

    pub fn editor<'a>(content: &'a text_editor::Content, on_action: impl Fn(text_editor::Action) -> Message + 'static) -> Element<'a, Message> {
        text_editor(content)
            .on_action(on_action)
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
