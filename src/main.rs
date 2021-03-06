mod config;
mod probe;
mod util;

use crate::probe::{ui, App, ZMQInput};
#[allow(dead_code)]
use crate::util::event::{Config, Event, Events};
use config::{Cli, Probes};

use std::fs;
use std::{error::Error, io, time::Duration};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, Terminal};

fn main() -> Result<(), Box<dyn Error>> {
    // get config
    let cli: Cli = argh::from_env();
    let config = fs::read_to_string(cli.config).expect("Something went wrong reading the file");
    let probes: Probes = toml::from_str(&config).unwrap();
    // println!("{:?}", probes);

    // set up terminal
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let inputs = vec![ZMQInput {
        name: "Probe 1".to_string(),
    }];

    // set up events and app
    let events = Events::with_config_and_probes(Config::default(), inputs);
    let mut app = App::new("Probe");
    app.probes = probes.probes;

    // event loop
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        match events.next()? {
            Event::Input(key) => match key {
                Key::Char(c) => {
                    app.on_key(c);
                }
                Key::Up => {
                    app.on_up();
                }
                Key::Down => {
                    app.on_down();
                }
                Key::Left => {
                    app.on_left();
                }
                Key::Right => {
                    app.on_right();
                }
                _ => {}
            },
            Event::Message(name, msg) => {
                app.process_message_for_stream(name, msg);
            }
        }
        if app.should_quit {
            break;
        }
    }

    Ok(())
}
