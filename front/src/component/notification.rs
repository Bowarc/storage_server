use std::time::Duration;

use {
    gloo::console::log,
    std::{cell::RefCell, sync::Mutex},
    yew::Callback,
};

// thread local: https://discord.com/channels/273534239310479360/1120124565591425034/1259034522888966164
thread_local! {
    // const: https://discord.com/channels/273534239310479360/1120124565591425034/1259038525823651870
    static CALLBACK: RefCell<Option<Callback<Notification>>> = const { RefCell::new(None) };
}
static CURRENT_ID: Mutex<u32> = Mutex::new(0);

pub fn push_notification(notification: Notification) {
    CALLBACK.with_borrow(|cb_opt| {
        let Some(cb) = cb_opt else {
            return;
        };
        cb.emit(notification)
    });
}

fn new_id() -> u32 {
    let mut guard = CURRENT_ID.lock().unwrap();
    *guard += 1;
    *guard - 1
}

pub enum Message {
    Push(Notification),
    RemoveAnimation { id: u32 },
    Remove { id: u32 },
}

#[derive(PartialEq)]
pub enum NotificationStyle{
    Info,
    Error,
}

#[derive(PartialEq)]
pub struct Notification {
    id: u32,
    expired: bool,
    timeout_s: f64,
    title: String,
    content: Vec<String>,
    style: NotificationStyle,
}

impl Notification {
    pub fn new(title: &str, content: Vec<&str>, timeout_s: f64, style: NotificationStyle) -> Self {
        Self {
            id: new_id(),
            expired: false,
            timeout_s,
            title: title.to_string(),
            content: content.iter().map(ToString::to_string).collect::<Vec<_>>(),
            style,
        }
    }
    pub fn info(title: &str, content: Vec<&str>, timeout_s: f64) -> Self{
        Self::new(title, content, timeout_s, NotificationStyle::Info)
    }

    pub fn error(title: &str, content: Vec<&str>, timeout_s: f64) -> Self{
        Self::new(title, content, timeout_s, NotificationStyle::Error)
    }
    fn update(&mut self) {
        // update time
    }

    fn render(&self) -> yew::Html {
        yew::html! {<div class={
                format!(
                    "notification{}{}",
                    if self.expired{" notification_expired"}else{""},
                    match self.style{
                        NotificationStyle::Info => " notification_info",
                        NotificationStyle::Error => " notification_error"
                    }
                )
            }>
            <div class="notification_title">{
                &self.title
            }</div>
            <div class="notification_content">{
                for self.content.iter().map(|bit|{
                    yew::html!{<>{
                        bit
                    }
                    <br />
                    </>}

                })
            }</div>
        </div>}
    }
}

pub struct NotificationManager {
    notifications: Vec<Notification>,
}

impl yew::Component for NotificationManager {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &yew::Context<Self>) -> Self {
        CALLBACK.set(Some(ctx.link().callback(Message::Push)));
        Self {
            notifications: Vec::new(),
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Push(notification) => {
                // log!("Added notification");

                ctx.link().send_future(async move {
                    gloo_timers::future::sleep(Duration::from_secs_f64(notification.timeout_s))
                        .await;
                    Message::RemoveAnimation {
                        id: notification.id,
                    }
                });
                self.notifications.push(notification);
            }
            Message::RemoveAnimation { id } => {
                let mut entries = self
                    .notifications
                    .iter_mut()
                    .filter(|n| n.id == id)
                    .collect::<Vec<_>>();

                if entries.len() != 1 {
                    log!("TODO: manage error: multiple notification with same id");
                }

                let notification = entries.get_mut(0).unwrap(); // This should never crash as we checked above
                notification.expired = true;
                ctx.link().send_future(async move {
                    gloo_timers::future::sleep(Duration::from_secs_f64(
                        0.1, /* This needs to be 1/10 of the css animation time, otherwise it leave a remnant image of the notification */
                    ))
                    .await;
                    Message::Remove { id }
                });
            }
            Message::Remove { id } => {
                // log!(format!("Removing {id}"));
                self.notifications.retain(|n| n.id != id);
            }
        }
        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        yew::html! {<div class="notification_block">{
            for self.notifications.iter().map(Notification::render)
        }</div>}
    }

    fn rendered(&mut self, ctx: &yew::prelude::Context<Self>, first_render: bool) {
        // log!("Notification re-rendered");
    }
}
