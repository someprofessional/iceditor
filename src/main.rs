use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use iced::{Font ,executor, Application, Command, Element, Length, Settings, Theme};
use iced::widget::{button, column, container, horizontal_space, row, text, text_editor, tooltip};
use iced::theme;
use iced::highlighter::{self, Highlighter};

fn main() -> iced::Result {
    Editor::run(Settings{
        default_font: Font::MONOSPACE,
        fonts: vec![include_bytes!("../fonts/editor-icons.ttf")
        .as_slice()
        .into()],
        ..Settings::default()
    })
}

struct Editor {
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    New,
    Open,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    Save,
    FileSaved(Result<PathBuf, Error>),
}

impl Application for Editor {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                path: None,
                content: text_editor::Content::new(),
                error: None,
            },
            Command::perform(
                load_file(default_file()),
                Message::FileOpened,
            ),
           
        )
    }


    fn title(&self) -> String {
        String::from("A cool editor !")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Edit(action) => {
                self.content.edit(action);
                self.error = None;
                Command::none()
            }
            Message::New => {
                self.path = None;
                self.content = text_editor::Content::new();
                
                Command::none()
            }
            Message::Open => Command::perform(pick_file(), Message::FileOpened),

            Message::FileOpened(Ok((path, content))) => {
                self.path = Some(path);
                self.content = text_editor::Content::with(&content);

                Command::none()
            }
            Message::FileOpened(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }
            Message::Save => {
                let text = self.content.text();

                Command::perform(save_file(self.path.clone(), text), Message::FileSaved)
            }
            Message::FileSaved(Ok(path)) => {
                self.path = Some(path);
                Command::none()
            } ,
            Message::FileSaved(Err(error)) => {
                self.error = Some(error);

                Command::none()
            }
        }

    }

    fn view(&self) -> Element<'_, Message> {
        let controls = row![
            action(new_icon(), "New file", Message::New),
            action(open_icon(), "Open file", Message::Open),
            action(save_icon(), "Save file", Message::Save)
        ].spacing(10).into();

        let input = text_editor(&self.content).on_edit(Message::Edit).highlight::<Highlighter>(highlighter::Settings {
            theme: highlighter::Theme::SolarizedDark,
            extension: self.path.as_ref().and_then(|path| path.extension()?.to_str()).unwrap_or("rs").to_string(),
        }, |highlight, _theme| {
            highlight.to_format()
        }).into();

        let status = if let Some(Error::IOFailed(error)) = self.error.as_ref() {
            text(error.to_string())
        } else {
            match self.path.as_deref().and_then(Path::to_str) {
            Some(path) => text(path).size(14),
            None => text("New file")
            }
        };
        
        let position: Element<'_, Message> = {
            let (line, column) = self.content.cursor_position();
            text(format!("{}:{}", line + 1, column + 1)).into() // Explicit `Element` type annotation
        };

        let status_bar = row![
            status, 
            horizontal_space(Length::Fill), position].into();
            container(column(vec![controls, input, status_bar]).spacing(10)).padding(10).into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

fn action<'a>(
    content: Element<'a, Message>, 
    label: &str,
    on_press: Message
) -> Element<'a, Message> {
    tooltip(
        button(container(content).width(30).center_x())
        .on_press(on_press) 
        .padding([5, 10]),
    label, 
    tooltip::Position::FollowCursor,
    )
    .style(theme::Container::Box)
    .into()
}

fn new_icon<'a>() -> Element<'a, Message> {
    icon('\u{E800}')
}

fn save_icon<'a>() -> Element<'a, Message> {
    icon('\u{E801}')
}

fn open_icon<'a>() -> Element<'a, Message> {
    icon('\u{F114}')
}

fn icon<'a>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");

    text(codepoint).font(ICON_FONT).into()
}

fn default_file() -> PathBuf {
    PathBuf::from(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")))
}

async fn pick_file() -> Result<(PathBuf, Arc<String>), Error> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Choose a text file ...")
        .pick_file()
        .await
        .ok_or(Error::DialogClosed)?;

    load_file(handle.path().to_owned()).await
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {
    let contents = tokio::fs::read_to_string(&path)
        .await
        .map(Arc::new)
        .map_err(|error| error.kind())
        .map_err(Error::IOFailed)?;
    
    Ok((path, contents))
}

async fn save_file(path: Option<PathBuf>, text: String) -> Result<PathBuf, Error>{
    let path = if let Some(path) = path { 
        path
     } else {
        rfd::AsyncFileDialog::new()
            .set_title("choose a file name ...")
            .save_file()
            .await
            .ok_or(Error::DialogClosed).map(|handle| handle.path().to_owned())?
    };

    tokio::fs::write(&path, &text).await.map_err(|error| Error::IOFailed(error.kind()))?;

    Ok(path)
}

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IOFailed(io::ErrorKind),
}