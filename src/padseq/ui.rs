use super::midi::{Instrument, MidiMessageType};
use super::sequencer::{Sequencer, WaitResult, NUMBER_OF_INSTRUMENTS};
use super::session::{Note, Step, StepNotes, BAR_SIZE};
use std::cmp;
use std::collections::HashSet;
use std::thread::sleep;
use std::time::{Duration, Instant};

const PAD_BAR_NOTES: [Note; BAR_SIZE as usize] = [
    81, 82, 83, 84, 85, 86, 87, 88, 71, 72, 73, 74, 75, 76, 77, 78, 61, 62, 63, 64, 65, 66, 67, 68,
    51, 52, 53, 54, 55, 56, 57, 58,
];
const PAD_KEY_NOTES: [Note; 12] = [22, 32, 23, 33, 24, 25, 35, 26, 36, 27, 37, 28];
const PAD_NEXT_CC: u8 = 94;
const PAD_PREV_CC: u8 = 93;
const PAD_COLOR_STEP_OFF: u8 = 112;
const PAD_COLOR_STEP_SET: u8 = 53;
const PAD_COLOR_STEP_SET_OTHER_NOTE: [u8; 4] = [19, 22, 17, 16];
const PAD_COLOR_STEP_SET_AND_ACTIVE: u8 = 78;
const PAD_COLOR_STEP_ACTIVE: u8 = 3;
const PAD_COLOR_KEY: u8 = 12;
const PAD_COLOR_KEY_ACTIVE: u8 = 9;
const BASE_C_NOTE: Note = 60;

type SelectedNotes = HashSet<Note>;

pub struct UI {
    sequencer: Sequencer,
    pad: Instrument,
    selected_notes: SelectedNotes,
    active_instrument: usize,
}

impl UI {
    pub fn new(sequencer: Sequencer) -> UI {
        UI {
            sequencer: sequencer,
            pad: Instrument::new(&"Pad".to_string()),
            selected_notes: SelectedNotes::new(),
            active_instrument: 0,
        }
    }

    pub fn handle_pad_events(&mut self) {
        while self.pad.has_events() {
            let event = self.pad.pop_event().unwrap();
            let message = event.message;
            let note = message.note;
            match message.r#type {
                MidiMessageType::ControlChange => {
                    if message.velocity > 0 {
                        match message.note {
                            PAD_NEXT_CC => {
                                self.active_instrument =
                                    cmp::min(NUMBER_OF_INSTRUMENTS, self.active_instrument + 1);
                            }
                            PAD_PREV_CC => {
                                self.active_instrument = cmp::max(0, self.active_instrument - 1);
                            }
                            _ => {}
                        }
                    }
                    println!("{} is new active instrument", self.active_instrument);
                }
                _ => {
                    if PAD_KEY_NOTES.contains(&note) {
                        let key_note = BASE_C_NOTE
                            + PAD_KEY_NOTES.iter().position(|&x| x == note).unwrap() as Note;
                        match message.r#type {
                            MidiMessageType::NoteOn => {
                                self.sequencer
                                    .get_instrument(self.active_instrument)
                                    .play_note(1, key_note, message.velocity, 0.0);
                                self.pad.play_note(1, note, PAD_COLOR_KEY_ACTIVE, 0.0);
                                match message.velocity {
                                    0 => {
                                        self.selected_notes.remove(&key_note);
                                    }
                                    _ => {
                                        self.selected_notes.insert(key_note);
                                    }
                                }
                            }
                            MidiMessageType::NoteOff => {
                                self.sequencer
                                    .get_instrument(self.active_instrument)
                                    .stop_note(1, key_note);
                                self.pad.play_note(1, note, PAD_COLOR_KEY, 0.0);
                                self.selected_notes.remove(&key_note);
                            }
                            _ => {
                                panic!()
                            }
                        }
                    } else if PAD_BAR_NOTES.contains(&note) {
                        if message.velocity > 0 {
                            let step =
                                PAD_BAR_NOTES.iter().position(|&x| x == note).unwrap() as Step;
                            if self.selected_notes.len() == 0 {
                                self.sequencer
                                    .get_session()
                                    .get_instrument(self.active_instrument)
                                    .get_pattern(0)
                                    .clear_step(step);
                            } else {
                                let mut step_notes = if self
                                    .sequencer
                                    .get_session()
                                    .get_instrument(self.active_instrument)
                                    .get_pattern(0)
                                    .has_step_set(step)
                                {
                                    self.sequencer
                                        .get_session()
                                        .get_instrument(self.active_instrument)
                                        .get_pattern(0)
                                        .get_step(step)
                                        .clone()
                                } else {
                                    StepNotes::new()
                                };
                                for note in &self.selected_notes {
                                    if step_notes.contains_key(note) {
                                        step_notes.remove(note);
                                    } else {
                                        step_notes.insert(*note, 127);
                                    }
                                }
                                self.sequencer
                                    .get_session()
                                    .get_instrument(self.active_instrument)
                                    .get_pattern(0)
                                    .set_step(step, &step_notes);
                            }
                            self.sequencer.save_session();
                        }
                    }
                }
            }
        }
        self.pad.send_events();
    }

    fn refresh_step(&mut self, step: Step, active_step: Step) {
        let note = PAD_BAR_NOTES[step as usize];
        let channel = if step == 0 || step == BAR_SIZE - 1 {
            3
        } else {
            1
        };
        if step == active_step {
            if self
                .sequencer
                .get_session()
                .get_instrument(self.active_instrument)
                .get_pattern(0)
                .has_step_set(step)
            {
                self.pad
                    .play_note(channel, note, PAD_COLOR_STEP_SET_AND_ACTIVE, 0.0);
            } else {
                self.pad
                    .play_note(channel, note, PAD_COLOR_STEP_ACTIVE, 0.0);
            }
        } else {
            if self
                .sequencer
                .get_session()
                .get_instrument(self.active_instrument)
                .get_pattern(0)
                .has_step_set(step)
            {
                let mut any_missing = self.selected_notes.len() == 0;
                for note in self.selected_notes.clone() {
                    if !self
                        .sequencer
                        .get_session()
                        .get_instrument(self.active_instrument)
                        .get_pattern(0)
                        .get_step(step)
                        .contains_key(&note)
                    {
                        any_missing = true;
                        break;
                    }
                }
                if !any_missing {
                    self.pad.play_note(channel, note, PAD_COLOR_STEP_SET, 0.0);
                } else {
                    self.pad.play_note(
                        channel,
                        note,
                        PAD_COLOR_STEP_SET_OTHER_NOTE[cmp::min(
                            cmp::max(
                                self.sequencer
                                    .get_session()
                                    .get_instrument(self.active_instrument)
                                    .get_pattern(0)
                                    .get_step(step)
                                    .len(),
                                1,
                            ) - 1,
                            3,
                        )],
                        0.0,
                    );
                }
            } else {
                self.pad.play_note(channel, note, PAD_COLOR_STEP_OFF, 0.0);
            }
        }
    }

    pub fn refresh(&mut self, active_step: Step) {
        for n in 0..BAR_SIZE {
            self.refresh_step(n, active_step);
        }
    }

    pub fn clear_highlighted_notes(&mut self) {
        for n in 0..12 {
            let note = PAD_KEY_NOTES[n];
            self.pad.play_note(1, note, PAD_COLOR_KEY, 0.0);
        }
    }

    pub fn highlight_played_note(&mut self, instrument: usize, note: Note) {
        if instrument == self.active_instrument {
            let key_note = PAD_KEY_NOTES[(note - BASE_C_NOTE) as usize];
            self.pad.play_note(1, key_note, PAD_COLOR_KEY_ACTIVE, 0.0);
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
                    self.clear_highlighted_notes();
                    let played_notes = self.sequencer.process_step();
                    for (instrument, note) in played_notes {
                        self.highlight_played_note(instrument, note);
                    }
                    let active_step = self.sequencer.get_active_step();
                    self.refresh(active_step);
                }
                WaitResult::Intermediate => {
                    self.handle_pad_events();
                }
            }
            sleep(Duration::from_micros(1));
        }
    }
}
