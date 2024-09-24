mod excel;
mod mail;
mod html;

use iced::widget::{button, checkbox, column, container, responsive, row, scrollable, text, text_input};
use iced::{alignment, theme, window, Application, Color, Command, Length, Renderer, Settings, Size, Theme};
use iced::{Element};
use iced::Alignment::Center;
use iced::widget::text_input::Id;

use iced_table::table;
use lettre::Transport;
use lettre::transport::smtp::authentication::Credentials;
use native_dialog::{MessageDialog, MessageType};
use crate::html::generate_html;
use crate::mail::send_mail;

pub const TOKEN: &'static str = "pctyuktfvtvybbhi";

#[derive(Debug)]
enum Mailbox {
    Config(AuthState),
    Main(State),
}

#[derive(Debug, Clone)]
struct State {
    list: Vec<Tasks>,
    headers: Vec<Header>,
    search_value: String,
    header: scrollable::Id,
    body: scrollable::Id,
    title: String,
    remark: String,
    auth: AuthState,
}

#[derive(Debug, Default, Clone)]
struct AuthState {
    username: String,
    password: String,
}

#[derive(Debug, Clone)]
struct Header {
    name: String,
    width: f32,
    check: bool,
}

#[derive(Debug, Default, Clone)]
struct Tasks {
    email: String,
    name: String,
    seq: i64,
    info: Vec<String>,
    status: bool,
}


#[derive(Debug, Clone)]
enum Message {
    Loaded(Result<SavedState, LoadError>),
    Username(String),
    Password(String),
    Search(String),
    SavePassword(bool),
    Login,
    Import,
    Title(String),
    Remark(String),
    Send,
    SyncHeader(scrollable::AbsoluteOffset),
    Enable(usize, bool),
    AllSelect(bool),
    Nop,
}
#[derive(Debug, Clone)]
struct SavedState {}
#[derive(Debug, Clone)]
enum LoadError {}

fn main() -> iced::Result {
    Mailbox::run(Settings {
        window: window::Settings {
            size: Size::new(1600.0, 800.0),
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

impl Application for Mailbox {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self::Config(AuthState::default()),
            Command::perform(SavedState::load(), Message::Loaded),
        )
    }

    fn title(&self) -> String {
        match self {
            Mailbox::Config(_) => "邮件配置信息".to_owned(),
            Mailbox::Main(_) => "邮件发送界面".to_owned(),
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match self {
            Mailbox::Config(state) => {
                match message {
                    Message::Loaded(result) => {
                        result.unwrap();
                    }
                    Message::Username(username) => {
                        state.username = username;
                    }
                    Message::Password(password) => {
                        state.password = password;
                    }
                    Message::Login => {
                        if state.username.is_empty() || state.password.is_empty() {
                            return Command::none();
                        }
                        if !mail::test(&state.username, &state.password) {
                            MessageDialog::new()
                                .set_type(MessageType::Error)
                                .set_title("登陆提示")
                                .set_text("账户名或密码有误，登陆失败！")
                                .show_alert()
                                .unwrap();
                            return Command::none();
                        }
                        *self = Mailbox::Main(State {
                            list: vec! {},
                            headers: vec![],
                            search_value: "".to_owned(),
                            header: scrollable::Id::unique(),
                            body: scrollable::Id::unique(),
                            remark: "".to_owned(),
                            title: "".to_owned(),
                            auth: state.clone(),
                        });
                    }
                    _ => {}
                }
            }
            Mailbox::Main(state) => {
                match message {
                    Message::Title(value) => {
                        state.title = value.trim().to_owned();
                    }
                    Message::Remark(value) => {
                        state.remark = value.trim().to_owned();
                    }
                    Message::Import => {
                        let file = rfd::FileDialog::new()
                            .add_filter("excel files (*.xlsx)", &["xlsx", "xls"])
                            .set_directory("/")
                            .pick_file();
                        if let Some(path) = file {
                            let (list, headers) = excel::parse_excel(path);
                            state.list = list;
                            state.headers = headers;
                        }
                    }
                    Message::Enable(row_index, enable) => {
                        state.list[row_index].status = enable;
                    }
                    Message::AllSelect(enable) => {
                        state.list.iter_mut().for_each(|task| task.status = enable);
                        state.headers.iter_mut().for_each(|header| header.check = enable);
                    }
                    Message::SyncHeader(offset) => {
                        return Command::batch(vec![
                            scrollable::scroll_to(state.header.clone(), offset)
                        ])
                    }
                    Message::Send => {
                        if state.list.is_empty() {
                            return Command::none();
                        }
                        let yes = MessageDialog::new()
                            .set_type(MessageType::Info)
                            .set_title("发送确认")
                            .set_text("是否确认发送?")
                            .show_confirm()
                            .unwrap();
                        if !yes {
                            return Command::none();
                        }
                        let creds = Credentials::new(state.auth.username.clone(), state.auth.password.clone());
                        for task in &state.list {
                            if task.status {
                                let html = generate_html(task, &state.headers, &state.remark);
                                send_mail(&state.title, &html, &format!("{}@wondersgroup.com", state.auth.username), &task.email, creds.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme, Renderer> {
        match self {
            Mailbox::Config(state) => {
                let title = text("登录")
                    .width(Length::Fill)
                    .size(50)
                    .horizontal_alignment(alignment::Horizontal::Center);

                let username = text_input("账户名", &state.username)
                    .id(Id::new("username"))
                    .on_input(Message::Username)
                    // .on_submit(Message::CreateTask)
                    .padding(30)
                    .size(20);

                let password = text_input("密码", &state.password)
                    .id(Id::new("password"))
                    .on_input(Message::Password)
                    .secure(true)
                    .padding(30)
                    .size(20);

                let password_check = checkbox("保存密码", true)
                    .on_toggle(Message::SavePassword);

                let btn = container(button("登录")
                    .padding([5, 10])
                    .on_press(Message::Login)
                    .style(theme::Button::Primary)).width(Length::Fill).center_y().center_x();

                let content = column![title, username, password, password_check, btn]
                    .spacing(20)
                    .max_width(500);

                scrollable(container(content).center_x().padding(40)).into()
            }
            Mailbox::Main(state) => {
                let import_button = button("导入Excel").padding([5, 10])
                    .on_press(Message::Import)
                    .style(theme::Button::Primary);

                let send_button = button("发送邮件").padding([5, 10])
                    .on_press(Message::Send)
                    .style(theme::Button::Primary);
                let title = row![text_input("邮件主题配置", &state.title).on_input(Message::Title).padding(10).size(20).width(1300), import_button].spacing(20);
                let remark = row![text_input("邮件提示信息", &state.remark).on_input(Message::Remark).padding(10).size(20).width(1300), send_button].spacing(20);

                let table: Element<_> = if state.list.is_empty() {
                    empty_message("请先导入Excel数据!")
                } else {
                    responsive(|size| {
                        table(
                            state.header.clone(),
                            state.body.clone(),
                            &state.headers,
                            &state.list,
                            Message::SyncHeader,
                        ).into()
                    }).into()
                };

                let content = column![title, remark, table]
                    .spacing(10);

                // scrollable(container(content).center_x(Fill).padding(40)).into()
                container(container(content).width(Length::Fill).height(Length::Fill))
                    .padding(20)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
                    .into()
            }
        }
    }
}

fn empty_message(message: &str) -> Element<'_, Message> {
    container(
        text(message)
            .width(Length::Fill)
            .size(25)
            .horizontal_alignment(alignment::Horizontal::Center)
            .style(Color::from([0.7, 0.7, 0.7])),
    )
        .height(200)
        .center_y()
        .into()
}

impl SavedState {
    async fn load() -> Result<SavedState, LoadError> {
        Ok(SavedState {})
    }
}

impl Tasks {
    fn view(&self) -> Element<'_, Message> {
        let checkbox = checkbox("", true)
            // .on_toggle(TaskMessage::Completed)
            .size(17)
            .text_shaping(text::Shaping::Advanced);
        let email = text(&self.email);
        let row = row![
                    checkbox,
            email,
            text(&self.seq),
            text(&self.name),
                    button("发送")
                        // .on_press()
                        .padding(10)
            .style(theme::Button::Text),
                ]
            .spacing(20)
            .into();
        row
    }

    fn at(&self, index: usize) -> String {
        self.info.get(index).map(|s| s.to_string()).unwrap_or_default()
    }
}


impl<'a> table::Column<'a, Message, Theme, Renderer> for Header {
    type Row = Tasks;

    fn header(&'a self, col_index: usize) -> Element<'a, Message> {
        if col_index == 0 {
            container(checkbox("", self.check).on_toggle(Message::AllSelect)).height(24).center_y().into()
        } else {
            println!("{}", &self.name);
            container(text(&self.name)).height(24).center_y().into()
        }
    }

    fn cell(&'a self, col_index: usize, row_index: usize, row: &'a Self::Row) -> Element<'a, Message> {
        let content: Element<_> = if col_index == 0 {
            checkbox("", row.status).on_toggle(move |enable| Message::Enable(row_index, enable)).into()
        } else if col_index == 1 {
            text(&row.email).into()
        } else if col_index == 2 {
            text(&row.seq).into()
        } else if col_index == 3 {
            text(&row.name).into()
        } else {
            text(&row.at(col_index - 4)).into()
        };
        container(content)
            .width(Length::Fill)
            .height(32)
            .center_y()
            .into()
    }

    fn width(&self) -> f32 {
        self.width
    }

    fn resize_offset(&self) -> Option<f32> {
        None
    }
}