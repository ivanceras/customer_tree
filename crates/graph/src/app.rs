use sauron::*;

pub enum Msg{
}
pub struct App{
}

impl Application for App{
    type MSG = Msg;

    fn update(&mut self, _msg: Msg) -> Cmd<Msg> {
        Cmd::none()
    }

    fn view(&self) -> Node<Msg> {
        node!{
            <svg viewBox="-100 -100 200 200" xmlns="http://www.w3.org/2000/svg">
              <circle cx="50" cy="50" r="4" fill="lightblue" stroke="blue" />
            </svg>
        }
    }
}
