use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use iced::{executor, Application, Command, Element, Length, Settings, Theme};
use iced::widget::{column, button, container, horizontal_space, row, text, text_editor};
fn main() -> iced::Result {
    Editor::run(Settings::default())
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
        }

    }

    fn view(&self) -> Element<'_, Message> {
        let controls = row![
            button("New").on_press(Message::New),
            button("Open").on_press(Message::Open)].into();

        let input = text_editor(&self.content).on_edit(Message::Edit).into();

        let file_path = match self.path.as_deref().and_then(Path::to_str) {
            Some(path) => text(path).size(14),
            None => text("New file")
        };
        
        let position: Element<'_, Message> = {
            let (line, column) = self.content.cursor_position();
            text(format!("{}:{}", line + 1, column + 1)).into() // Explicit `Element` type annotation
        };

        let status_bar = row![
            file_path, 
            horizontal_space(Length::Fill), position].into();
            container(column(vec![controls, input, status_bar]).spacing(10)).padding(10).into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
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
        .map_err(Error::IO)?;
    
    Ok((path, contents))
}

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IO(io::ErrorKind),
}