pub mod screens;

use super::midi::Instrument;
use super::sequencer::{Sequencer, WaitResult};
use crate::padseq::session::{Note, Pattern as SessionPattern};
use screens::pattern::Pattern;
use screens::session::Session;
use std::thread::sleep;
use std::time::Duration;

pub enum ScreenEvent {
    None,
    SwitchToPattern(usize, usize),
    SwitchToSession,
}

pub struct UIContext<'a> {
    pad: &'a mut Instrument,
    sequencer: &'a mut Sequencer,
}

macro_rules! create_context {
    ($ui:ident) => {
        &mut UIContext {
            pad: &mut $ui.pad,
            sequencer: &mut $ui.sequencer,
        }
    };
}

pub trait Screen {
    fn handle_pad_events(&mut self, context: &mut UIContext) -> ScreenEvent;
    fn refresh(&mut self, context: &mut UIContext);
    fn prepare_step(&mut self, _context: &mut UIContext) {}
    fn on_played_note(&mut self, _context: &mut UIContext, _instrument: usize, _note: Note) {}
    fn clear(&mut self, _context: &mut UIContext) {}
}

pub struct UI {
    sequencer: Sequencer,
    pad: Instrument,
    screen: Box<dyn Screen>,
}

impl UI {
    pub fn new(sequencer: Sequencer) -> UI {
        UI {
            sequencer: sequencer,
            pad: Instrument::new(&"Pad".to_string()),
            screen: Box::new(Session::new()),
        }
    }

    pub fn run(&mut self) {
        print!("run");
        self.pad.connect_out(2);
        self.pad.connect_in(2);
        self.sequencer.connect();
        print!("Connect done");
        loop {
            match self.sequencer.wait() {
                WaitResult::Step => {
                    self.screen.prepare_step(create_context!(self));
                    let played_notes = self.sequencer.process_step();
                    for (instrument, note) in played_notes {
                        self.screen
                            .on_played_note(create_context!(self), instrument, note);
                    }
                    self.screen.refresh(create_context!(self));
                }
                WaitResult::Intermediate => {
                    match self.screen.handle_pad_events(create_context!(self)) {
                        ScreenEvent::SwitchToSession => {
                            self.screen.clear(create_context!(self));
                            self.screen = Box::new(Session::new());
                        }
                        ScreenEvent::SwitchToPattern(instrument, pattern) => {
                            println!("switch to {} {}", instrument, pattern);
                            if !self
                                .sequencer
                                .get_session()
                                .get_instrument(instrument)
                                .has_pattern(pattern)
                            {
                                self.sequencer
                                    .get_session_mut()
                                    .get_instrument_mut(instrument)
                                    .set_pattern(pattern, &SessionPattern::new());
                            }
                            self.screen.clear(create_context!(self));
                            self.screen = Box::new(Pattern::new(instrument, pattern));
                        }
                        ScreenEvent::None => {}
                    }
                }
            }
            sleep(Duration::from_micros(1));
        }
    }
}
