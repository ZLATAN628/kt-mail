mod excel;
mod mail;
mod html;

use std::fs;
use iced::widget::{button, checkbox, column, container, responsive, row, scrollable, text, text_input};
use iced::{alignment, event, keyboard, theme, window, Application, Color, Command, Event, Length, Renderer, Settings, Size, Subscription, Theme};
use iced::{Element};
use iced::keyboard::key;
use iced::widget::text_input::Id;

use iced_table::table;
use lettre::Transport;
use lettre::transport::smtp::authentication::Credentials;
use native_dialog::{MessageDialog, MessageType};
use serde::{Deserialize, Serialize};
use crate::html::generate_html;
use crate::mail::send_mail;

pub const SAVED_FILE: &'static str = "./auth.dll";
pub const MAIL_FILE: &'static str = "./mail.dll";


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
    send_message: String,
    cur_page: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct MailData {
    title: String,
    remark: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct AuthState {
    username: String,
    password: String,
    save: bool,
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
    Loaded(AuthState),
    Username(String),
    Password(String),
    Save(bool),
    Login,
    Import,
    Title(String),
    Remark(String),
    Send,
    SyncHeader(scrollable::AbsoluteOffset),
    Enable(usize, bool),
    AllSelect(bool),
    NextPage,
    PrevPage,
    BeginSend,
    EndSend(Vec<Tasks>),
    Event(Event),
    Nop,
}

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
            Self::Config(AuthState {
                save: true,
                ..AuthState::default()
            }),
            Command::perform(AuthState::load(), Message::Loaded),
        )
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        event::listen().map(Message::Event)
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
                    Message::Loaded(auth) => {
                        *state = auth;
                    }
                    Message::Username(username) => {
                        state.username = username;
                    }
                    Message::Password(password) => {
                        state.password = password;
                    }
                    Message::Save(save) => {
                        state.save = save;
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

                        if state.save {
                            let data = serde_json::to_string(&state).unwrap();
                            fs::write(SAVED_FILE, &data).unwrap();
                        } else {
                            fs::remove_file(SAVED_FILE).unwrap();
                        }

                        let mail_data = read_mail_data();
                        *self = Mailbox::Main(State {
                            list: vec! {},
                            headers: vec![],
                            search_value: "".to_owned(),
                            header: scrollable::Id::unique(),
                            body: scrollable::Id::unique(),
                            remark: mail_data.remark,
                            title: mail_data.title,
                            auth: state.clone(),
                            cur_page: 0,
                            send_message: String::new(),
                        });
                    }
                    Message::Event(event) => match event {
                        Event::Keyboard(keyboard::Event::KeyPressed {
                                            key: keyboard::Key::Named(key::Named::Enter),
                                            ..
                                        }) => {
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

                            if state.save {
                                let data = serde_json::to_string(&state).unwrap();
                                fs::write(SAVED_FILE, &data).unwrap();
                            } else {
                                fs::remove_file(SAVED_FILE).unwrap();
                            }

                            let mail_data = read_mail_data();
                            *self = Mailbox::Main(State {
                                list: vec! {},
                                headers: vec![],
                                search_value: "".to_owned(),
                                header: scrollable::Id::unique(),
                                body: scrollable::Id::unique(),
                                remark: mail_data.remark,
                                title: mail_data.title,
                                auth: state.clone(),
                                cur_page: 0,
                                send_message: String::new(),
                            });
                        }
                        _ => {}
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
                    Message::BeginSend => {
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
                        state.send_message = "发送邮件中...".to_owned();

                        return Command::perform(State::send(state.clone()), Message::EndSend);
                    }
                    Message::EndSend(tasks) => {
                        if tasks.is_empty() {
                            state.send_message = "发送完毕".to_string();
                        } else {
                            state.send_message = format!("发送完毕，剩余{}条邮件未发送成功", tasks.len());
                        }
                        state.list = tasks;

                        set_mail_data(&MailData {
                            remark: state.remark.clone(),
                            title: state.title.clone(),
                        });
                    }
                    Message::PrevPage => {
                        if state.cur_page > 0 {
                            state.cur_page -= 1;
                        }
                    }
                    Message::NextPage => {
                        if state.cur_page + 1 < (state.list.len() + 49) / 50 {
                            state.cur_page += 1;
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

                let password_check = checkbox("保存密码", state.save)
                    .on_toggle(Message::Save);

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
                    .on_press(Message::BeginSend)
                    .style(theme::Button::Primary);

                let send_info = if state.send_message.is_empty() {
                    text("")
                } else {
                    text(&state.send_message)
                };
                let title = row![text_input("邮件主题配置", &state.title).on_input(Message::Title).padding(10).size(20).width(1100), import_button].spacing(20);
                let remark = row![text_input("邮件提示信息", &state.remark).on_input(Message::Remark).padding(10).size(20).width(1100), send_button, send_info].spacing(20);

                let prev_button = button("上一页").padding([5, 10])
                    .on_press(Message::PrevPage)
                    .style(theme::Button::Secondary);

                let next_button = button("下一页").padding([5, 10])
                    .on_press(Message::NextPage)
                    .style(theme::Button::Secondary);

                let page_info = text(format!("第 {}/{} 页", state.cur_page + 1, (state.list.len() + 49) / 50));
                let page_buttons = row![prev_button, next_button, page_info].spacing(20);

                let table: Element<_> = if state.list.is_empty() {
                    empty_message("请先导入Excel数据!")
                } else {
                    let next_index = (state.cur_page + 1) * 50;
                    let next_index = if next_index > state.list.len() {
                        state.list.len()
                    } else {
                        next_index
                    };
                    responsive(move |size| {
                        table(
                            state.header.clone(),
                            state.body.clone(),
                            &state.headers,
                            &state.list[state.cur_page * 50..next_index],
                            Message::SyncHeader,
                        ).into()
                    }).into()
                };

                let content = column![title, remark, page_buttons, table]
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

impl AuthState {
    async fn load() -> AuthState {
        match fs::read_to_string(SAVED_FILE) {
            Ok(data) => serde_json::from_str(&data).unwrap(),
            Err(_) => AuthState { save: true, ..AuthState::default() }
        }
    }
}

impl State {
    async fn send(state: State) -> Vec<Tasks> {
        let creds = Credentials::new(state.auth.username.clone(), state.auth.password.clone());
        let mut failed_task = vec![];
        for task in state.list {
            if task.status {
                let html = generate_html(&task, &state.headers, &state.remark);
                match send_mail(&format!("[{}]{}", &task.name, state.title), &html,
                                &format!("{}@wondersgroup.com", state.auth.username),
                                &task.email, creds.clone()) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("Error sending email to {}: {}", task.email, e);
                        failed_task.push(task.clone());
                    }
                }
            } else {
                failed_task.push(task.clone());
            }
        }

        failed_task
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

fn read_mail_data() -> MailData {
    match fs::read_to_string(MAIL_FILE) {
        Ok(data) => serde_json::from_str(&data).unwrap(),
        Err(_) => MailData::default()
    }
}

fn set_mail_data(data: &MailData) {
    fs::write(MAIL_FILE, serde_json::to_string(data).unwrap()).ok();
}