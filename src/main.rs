
use std::{io, fs, path::{Path, PathBuf}};
use tempfile::TempDir;

use iced::widget::{
    button, column, container, horizontal_space, row, text,
    text_editor, pick_list, scrollable
};
use iced::{
    Theme, Element, Length, Application, Settings, Command,
    Subscription, Event, Font
};
use iced::{alignment::Horizontal, executor, event, subscription, window};
use iced_highlighter::{self as highlighter, Highlighter};


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
        fs::write(path.join("Cargo.toml"), remove_extra_newline(self.toml)).unwrap();
        fs::create_dir(path.join("src")).unwrap();
        fs::write(path.join("src/lib.rs"), remove_extra_newline(self.lib_rs)).unwrap();
        fs::create_dir(path.join("tests")).unwrap();
        fs::write(path.join("tests/test.rs"), remove_extra_newline(self.test_rs)).unwrap();
    }

    fn make_new(&self) -> (text_editor::Content, text_editor::Content) {
        let src_code = text_editor::Content::with_text(unsafe { std::str::from_utf8_unchecked(self.lib_rs) });
        let test_code = text_editor::Content::with_text(unsafe { std::str::from_utf8_unchecked(self.test_rs) });

        (src_code, test_code)
    }

    fn read(path: &Path) -> (text_editor::Content, text_editor::Content) {
        let src_code = text_editor::Content::with_text(
            &fs::read_to_string(path.join("src/lib.rs")).unwrap()
        );
        let test_code = text_editor::Content::with_text(
            &fs::read_to_string(path.join("tests/test.rs")).unwrap()
        );

        (src_code, test_code)
    }
}

const WINDOWS_NEWLINE: u8 = b'\r';
const README_TEXT: &str = include_str!("../README.md");
const FRESH_ATTRIBUTE: Project = Project {
    toml: include_bytes!("../assets/attribute/Cargo.toml"),
    lib_rs: include_bytes!("../assets/attribute/src/lib.rs"),
    test_rs: include_bytes!("../assets/attribute/tests/test.rs"),
};
const FRESH_DECLARATIVE: Project = Project {
    toml: include_bytes!("../assets/declarative/Cargo.toml"),
    lib_rs: include_bytes!("../assets/declarative/src/lib.rs"),
    test_rs: include_bytes!("../assets/declarative/tests/test.rs"),
};
const FRESH_DERIVE: Project = Project {
    toml: include_bytes!("../assets/derive/Cargo.toml"),
    lib_rs: include_bytes!("../assets/derive/src/lib.rs"),
    test_rs: include_bytes!("../assets/derive/tests/test.rs"),
};
const FRESH_FUNCTION: Project = Project {
    toml: include_bytes!("../assets/function/Cargo.toml"),
    lib_rs: include_bytes!("../assets/function/src/lib.rs"),
    test_rs: include_bytes!("../assets/function/tests/test.rs"),
};

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

        FRESH_ATTRIBUTE.write(slf.attribute.path());
        FRESH_DECLARATIVE.write(slf.declarative.path());
        FRESH_DERIVE.write(slf.derive.path());
        FRESH_FUNCTION.write(slf.function.path());

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
    src_code: text_editor::Content,
    test_code: text_editor::Content,
    expansion: text_editor::Content,
    errors: text_editor::Content,
    macro_type: Option<MacroType>,
    temp_dirs: Option<TempDirs>,
    theme: highlighter::Theme,
    thinking: bool,
}

#[derive(Clone, Debug)]
pub enum Message {
    CloseRequested,
    EditExpansion(text_editor::Action),
    EditErrors(text_editor::Action),
    EditSource(text_editor::Action),
    EditTest(text_editor::Action),
    ExpandRequested,
    Expanded(Result<(String, String)>),
    Ignore,
    Navigate(Option<MacroType>),
    New,
    ThemeSelected(highlighter::Theme),
}

impl Application for Macaroni {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                src_code: text_editor::Content::new(),
                test_code: text_editor::Content::new(),
                expansion: text_editor::Content::new(),
                errors: text_editor::Content::new(),
                macro_type: None,
                temp_dirs: Some(TempDirs::new()),
                theme: highlighter::Theme::SolarizedDark,
                thinking: false,
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
            Message::EditErrors(action) => {
                self.errors.perform(action);
                Command::none()
            }
            Message::EditSource(action) => {
                self.src_code.perform(action);
                Command::none()
            }
            Message::EditTest(action) => {
                self.test_code.perform(action);
                Command::none()
            }
            Message::ExpandRequested => {
                self.thinking = true;
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
            Message::Expanded(Ok((expansion, errors))) => {
                self.thinking = false;
                self.expansion = text_editor::Content::with_text(&expansion);
                self.errors = text_editor::Content::with_text(&errors);
                Command::none()
            }
            Message::Expanded(Err(Error::CargoFailed(error))) => {
                println!("{}", &error);
                self.thinking = false;
                Command::none()
            }
            Message::Expanded(Err(_error)) => {
                self.thinking = false;
                Command::none()
            }
            Message::Ignore => Command::none(),
            Message::Navigate(macro_type) => {
                if macro_type != self.macro_type {
                    self.macro_type = macro_type;
                    if let Some(ref macro_type) = self.macro_type {
                        let dir = match macro_type {
                            MacroType::Attribute => self.temp_dirs.as_ref().unwrap().attribute.path(),
                            MacroType::Declarative => self.temp_dirs.as_ref().unwrap().declarative.path(),
                            MacroType::Derive => self.temp_dirs.as_ref().unwrap().derive.path(),
                            MacroType::Function => self.temp_dirs.as_ref().unwrap().function.path(),
                        };
                        (self.src_code, self.test_code) = Project::read(dir);
                        self.expansion = text_editor::Content::new();
                        self.errors = text_editor::Content::new();
                    }
                }
                Command::none()
            }
            Message::New => {
                match self.macro_type {
                    Some(MacroType::Attribute) => {
                        (self.src_code, self.test_code) = FRESH_ATTRIBUTE.make_new();
                    }
                    Some(MacroType::Declarative) => {
                        (self.src_code, self.test_code) = FRESH_DECLARATIVE.make_new();
                    }
                    Some(MacroType::Derive) => {
                        (self.src_code, self.test_code) = FRESH_DERIVE.make_new();
                    }
                    Some(MacroType::Function) => {
                        (self.src_code, self.test_code) = FRESH_FUNCTION.make_new();
                    }
                    None => {}
                }
                if self.macro_type.is_some() {
                    self.expansion = text_editor::Content::new();
                    self.errors = text_editor::Content::new();
                }
                Command::none()
            }
            Message::ThemeSelected(new_theme) => {
                self.theme = new_theme;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let title_bar = row![
            horizontal_space(Length::Fill),
            text("Macaroni"),
            horizontal_space(Length::Fill)
        ];
        let pickers = row![
            horizontal_space(Length::Fill),
            pick_list(highlighter::Theme::ALL, Some(self.theme), Message::ThemeSelected),
        ];

        let tabs = container(row![
                button("Home").on_press(Message::Navigate(None)),
                button("Attribute").on_press(Message::Navigate(Some(MacroType::Attribute))),
                button("Declarative").on_press(Message::Navigate(Some(MacroType::Declarative))),
                button("Derive").on_press(Message::Navigate(Some(MacroType::Derive))),
                button("Function").on_press(Message::Navigate(Some(MacroType::Function))),
            ]
            .spacing(10)
        );

        let controls = if self.macro_type.is_some() {
            row![
                container(button("New").on_press(Message::New))
                    .width(Length::Fill)
                    .align_x(Horizontal::Left),
                tabs,
                container(button("Expand").on_press(Message::ExpandRequested))
                    .width(Length::Fill)
                    .align_x(Horizontal::Right),
            ]
        } else {
            row![
                horizontal_space(Length::Fill),
                tabs,
                horizontal_space(Length::Fill),
            ]
            .width(Length::Fill)
        }
        .spacing(10);

        let src = elements::editor(&self.src_code, self.theme, Message::EditSource);
        let test = elements::editor(&self.test_code, self.theme, Message::EditTest);
        let ignore_edits = |action| match action {
            text_editor::Action::Edit(_) => Message::Ignore,
            action => Message::EditExpansion(action),
        };
        let expansion = elements::editor(&self.expansion, self.theme, ignore_edits);
        let errors = elements::editor(&self.errors, self.theme, ignore_edits);

        let body = if self.macro_type.is_some() {
            let column1 = column![
                text("Source code"),
                src,
                text("Test code"),
                test,
            ]
            .width(Length::Fill)
            .spacing(20);

            let column2 = if self.thinking {
                column![text("Expanded code"), text("Expanding..."), expansion, text("Errors and warnings"), errors]
            } else {
                column![text("Expanded code"), expansion, text("Errors and warnings"), errors]
            }
            .width(Length::Fill)
            .spacing(20);

            row![column1, column2].spacing(20)
        } else {
            row![scrollable(text(README_TEXT)).width(Length::Fill)]
            .width(Length::Fill)
        };

        let status_bar = elements::status_bar(self);

        container(
            column![
                title_bar,
                pickers,
                controls,
                body,
                status_bar
            ]
            .spacing(10)
        )
        .padding(10)
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Light
    }
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    DialogClosed,
    CargoFailed(String),
    IOFailed(io::ErrorKind)
}

fn remove_extra_newline(slice: &[u8]) -> Vec<u8> {
    slice.into_iter()
        .filter(|b| b != &&WINDOWS_NEWLINE)
        .copied()
        .collect()
}

pub mod actions {
    use super::*;

    pub async fn expand(dir: PathBuf, src_code: String, test_code: String) -> Result<(String, String)> {
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
                let expansion = std::str::from_utf8(&output.stdout).unwrap().to_owned();
                let errors = std::str::from_utf8(&output.stderr).unwrap().to_owned();
                Ok((expansion, errors))
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
        let status =             match slf.macro_type {
            Some(ref t) => text(format!("{:?}", t)),
            None => text("Home")
        };
        row![status, horizontal_space(Length::Fill)].into()
    }

    pub fn editor<'a>(content: &'a text_editor::Content, theme: highlighter::Theme, on_action: impl Fn(text_editor::Action) -> Message + 'static) -> Element<'a, Message> {
        text_editor(content)
            .on_action(on_action)
            .highlight::<Highlighter>(
            iced_highlighter::Settings {
                theme,
                extension: "rs".to_string()
            },
            |highlight, _theme| { highlight.to_format() }
        )
        .into()
    }
}
