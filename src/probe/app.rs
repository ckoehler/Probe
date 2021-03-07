use crate::probe::config::Probe;
use crate::probe::state::TabsState;

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub probes: Vec<Probe>,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, probes: Vec<Probe>) -> App<'a> {
        App {
            title,
            should_quit: false,
            tabs: TabsState::new(vec!["Tab0", "Tab1", "Tab2"]),
            probes: probes,
        }
    }

    pub fn on_up(&mut self) {}

    pub fn on_down(&mut self) {}

    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
        // self.probes[0].count += 1;
    }

    pub fn process_message_for_stream(&mut self, name: String, msg: String) {
        self.probes
            .iter_mut()
            .filter(|p| p.name == name)
            .for_each(|p: &mut Probe| p.count += 1);
    }
}
