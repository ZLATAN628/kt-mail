use iced::keyboard;
use iced::widget::{
    self, button, center, checkbox, column, container, keyed_column, row,
    scrollable, text, text_input, Text,
};
use iced::window;
use iced::{Center, Element, Fill, Font, Subscription, Task};
use iced::widget::text_input::Id;

use lettre::{Message as M2, SmtpTransport, Transport};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;

pub const TOKEN: &'static str = "pctyuktfvtvybbhi";

#[derive(Debug)]
enum Mailbox {
    Config,
    Main,
}

#[derive(Debug, Default)]
struct State {
    username: String,
    password: String,
    dirty: bool,
    saving: bool,
}


#[derive(Debug, Clone)]
enum Message {
    Loaded(Result<SavedState, LoadError>),
    Username(String),
    Password(String),
    SavePassword(bool),
    Nop,
}
#[derive(Debug, Clone)]
struct SavedState {}
#[derive(Debug, Clone)]
enum LoadError {}

fn main() -> iced::Result {
    // let email = M2::builder()
    //     .from("yuchenxing <yuchenxing@wondersgroup.com>".to_string().parse().unwrap())
    //     .to("ycx540188804 <ycx540188804@163.com>".parse().unwrap())
    //     .subject("Happy new year")
    //     .header(ContentType::TEXT_PLAIN)
    //     .body(String::from("Be happy 222!"))
    //     .unwrap();
    //
    // let creds = Credentials::new("yuchenxing".to_owned(), "Ycx19981118.".to_owned());
    // let mailer = SmtpTransport::builder_dangerous("smtp.wondersgroup.com")
    //     .credentials(creds)
    //     .build();
    //
    // match mailer.send(&email) {
    //     Ok(_) => println!("Email sent successfully!"),
    //     Err(e) => eprintln!("Error sending email: {}", e),
    // }

    iced::application(Mailbox::title, Mailbox::update, Mailbox::view)
        .window_size((1600.0, 800.0))
        .run_with(Mailbox::new)
}

impl Mailbox {
    fn new() -> (Self, Task<Message>) {
        (
            Self::Config,
            Task::perform(SavedState::load(), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        match self {
            Mailbox::Config => "邮件配置信息".to_owned(),
            Mailbox::Main => "邮件发送界面".to_owned(),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Loaded(result) => {
                result.unwrap();
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        match self {
            Mailbox::Config => {
                let title = text("Login")
                    .width(Fill)
                    .size(50)
                    .color([0.5, 0.5, 0.5])
                    .align_x(Center);

                let username = text_input("账户名", "")
                    .id(Id::new("username"))
                    .on_input(Message::Username)
                    // .on_submit(Message::CreateTask)
                    .padding(30)
                    .size(20)
                    .align_x(Center);
                let password = text_input("密码", "")
                    .id(Id::new("password"))
                    .on_input(Message::Password)
                    // .on_submit(Message::CreateTask)
                    .padding(30)
                    .size(20)
                    .align_x(Center);

                let password_check = checkbox("保存密码", true)
                    .on_toggle(Message::SavePassword);
                // let controls = view_controls(tasks, *filter);
                // let filtered_tasks =
                //     tasks.iter().filter(|task| filter.matches(task));

                // let tasks: Element<_> = if filtered_tasks.count() > 0 {
                //     keyed_column(
                //         tasks
                //             .iter()
                //             .enumerate()
                //             .filter(|(_, task)| filter.matches(task))
                //             .map(|(i, task)| {
                //                 (
                //                     task.id,
                //                     task.view(i).map(move |message| {
                //                         Message::TaskMessage(i, message)
                //                     }),
                //                 )
                //             }),
                //     )
                //         .spacing(10)
                //         .into()
                // } else {
                //     empty_message(match filter {
                //         Filter::All => "You have not created a task yet...",
                //         Filter::Active => "All your tasks are done! :D",
                //         Filter::Completed => {
                //             "You have not completed a task yet..."
                //         }
                //     })
                // };

                let content = column![title, username, password, password_check]
                    .spacing(20)
                    .max_width(500)
                    .align_x(Center);

                scrollable(container(content).center_x(Fill).padding(40)).into()
            }
            Mailbox::Main => { todo!() }
        }
    }
}

impl SavedState {
    async fn load() -> Result<SavedState, LoadError> {
        Ok(SavedState {})
    }
}