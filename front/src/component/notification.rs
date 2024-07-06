use std::{collections::VecDeque, sync::Mutex, time::Duration};

use gloo::console::log;
use yew::Callback;

static QUEUE: Mutex<VecDeque<Notif>> = Mutex::new(VecDeque::new());
const UPDATE_DELAY_MS: u64 = 1000;

pub static mut CALLBACK: Option<Callback<Notif>> = None;


pub fn push_notification(notif: Notif) {
    let mut guard = QUEUE.lock().unwrap();

    guard.push_back(notif);
}

pub enum Msg {
    Update,
    Push(Notif)
}

#[derive(PartialEq)]
pub struct Notif {
    pub content: String,
}

impl Notif{
    fn update(&mut self){
        // update time
    }

    fn render(&self) -> yew::Html{
        yew::html!{<div class="notification">{
            &self.content
        }</div>}
    }
}

pub struct Notification {
    notifs: Vec<Notif>,
}

impl yew::Component for Notification {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &yew::Context<Self>) -> Self {
        unsafe{
            CALLBACK = Some(ctx.link().callback(Msg::Push));
        }

        Self { notifs: Vec::new() }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Update => {
                log!("Notification update");
                ctx.link().send_future(async{
                    gloo_timers::future::sleep(Duration::from_millis(UPDATE_DELAY_MS)).await;
                    Msg::Update
                });
                let mut guard = QUEUE.lock().unwrap();
                while let Some(notif) = guard.pop_front(){
                    self.notifs.push(notif)
                }

                for notif in self.notifs.iter_mut(){
                    notif.update();
                }
            }
            Msg::Push(notif) => {
                log!("Added notification");
                self.notifs.push(notif)
            }
        }
        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {

        yew::html! {
            for self.notifs.iter().map(Notif::render)
        }
    }

    fn rendered(&mut self, ctx: &yew::prelude::Context<Self>, first_render: bool) {
        log!("Notification re-rendered");
    }


}
