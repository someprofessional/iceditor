use std::io;
use std::path::Path;
use std::sync::Arc;

use iced::{executor, Application, Command, Element, Length, Settings, Theme};
use iced::widget::{column, container, horizontal_space, row, text, text_editor};
fn main() -> iced::Result {
    Editor::run(Settings::default())
}

struct Editor {
    content: text_editor::Content,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    FileOpened(Result<Arc<String>, io::ErrorKind>),
}

impl Application for Editor {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                content: text_editor::Content::new(),
            },
            Command::perform(load_file(format!(
                "{}/src/main.rs",
                env!("CARGO_MANIFEST_DIR")
            )), Message::FileOpened),
           
        )
    }

    fn title(&self) -> String {
        String::from("A cool editor !")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Edit(action) => {
                self.content.edit(action);
            }
            Message::FileOpened(result) => {
                if let Ok(content) = result {
                    self.content = text_editor::Content::with(&content);
                }
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let input = text_editor(&self.content).on_edit(Message::Edit).into();

        let position: Element<'_, Message> = {
            let (line, column) = self.content.cursor_position();
            text(format!("{}:{}", line + 1, column + 1)).into() // Explicit `Element` type annotation
        };

        let status_bar = row![horizontal_space(Length::Fill), position].into();

        container(column(vec![input, status_bar]).spacing(10)).padding(10).into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

async fn load_file(path: impl AsRef<Path>) -> Result<Arc<String>, io::ErrorKind> {
    tokio::fs::read_to_string(path).await.map(Arc::new).map_err(|error| error.kind())
}