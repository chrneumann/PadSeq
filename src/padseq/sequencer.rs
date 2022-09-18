use super::midi::{Instrument, MidiMessageType};
use super::session::{Note, Session, Step};
use std::thread::sleep;
use std::time::Duration;

pub struct Sequencer {
    session: Session,
    pad: Instrument,
    instrument: Instrument,
    selected_note: Note,
    active: Step,
}

impl Sequencer {
    pub fn new() -> Sequencer {
        Sequencer {
            session: Session::new(),
            pad: Instrument::new("Pad"),
            instrument: Instrument::new("Instrument"),
            active: 0,
            selected_note: 0,
            // instruments: Vec<Instrument>::new(),
        }
    }

    pub fn connect(&mut self) {
        self.pad.connect_out(2);
        self.pad.connect_in(2);
        self.instrument.connect_out(0);
        print!("Connect done");
    }

    pub fn handle_pad_events(&mut self) {
        while self.pad.has_events() {
            let event = self.pad.pop_event().unwrap();
            let message = event.message;
            let note = message.note;
            if note >= 61 && note <= 68 {
                match message.r#type {
                    MidiMessageType::NoteOn => self.instrument.play_note(note, 127, 0),
                    MidiMessageType::NoteOff => self.instrument.stop_note(note),
                }
                self.selected_note = note;
            } else if note >= 81 && note <= 88 {
                if message.velocity > 0 {
                    let _note = if self.session.get_step(note - 81) > 0 {
                        0
                    } else {
                        self.selected_note
                    };
                    println!("set note {} {}", note, _note);
                    self.session.set_step(note - 81, _note);
                }
            }
        }
    }

    fn play_notes(&mut self) {
        let note = self.session.get_step(self.active);
        if note > 0 {
            println!("play {}", note);
            self.instrument.stop_note(note);
            self.instrument.play_note(note, 127, 150);
        }
    }

    fn refresh_step(&mut self, step: Step) {
        let note = 81 + step;
        if step == self.active {
            println!("{} is active", note);
            self.pad.play_note(note, 70, 0);
        } else {
            if self.session.get_step(step) > 0 {
                println!("{} is set", note);
                self.pad.play_note(note, 40, 0);
            } else {
                println!("{} is empty", note);
                self.pad.stop_note(note);
            }
        }
    }

    fn refresh(&mut self) {
        for n in 0..8 {
            self.refresh_step(n);
        }
    }

    pub fn run(&mut self) {
        let mut step = 0;
        loop {
            if step >= (150) {
                self.active = (self.active + 1) % 8;
                step = 0;
                self.play_notes();
                self.refresh();
            }
            self.instrument.send_events();
            self.handle_pad_events();
            self.pad.send_events();
            sleep(Duration::from_millis(10));
            step += 10;
        }
    }
}
