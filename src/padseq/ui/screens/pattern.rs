use crate::padseq::midi::MidiMessageType;
use crate::padseq::session::{Note, Step, StepNotes, BAR_SIZE};
use crate::padseq::ui::{Screen, ScreenEvent, UIContext};
use std::cmp;
use std::collections::HashSet;

pub const PAD_BAR_NOTES: [Note; BAR_SIZE as usize] = [
    81, 82, 83, 84, 85, 86, 87, 88, 71, 72, 73, 74, 75, 76, 77, 78, 61, 62, 63, 64, 65, 66, 67, 68,
    51, 52, 53, 54, 55, 56, 57, 58,
];
const PAD_KEY_NOTES: [Note; 13] = [21, 22, 32, 23, 33, 24, 25, 35, 26, 36, 27, 37, 28];
const PAD_PREV_OCTAVE: u8 = 31;
const PAD_NEXT_OCTAVE: u8 = 38;
const PAD_SESSION_CC: u8 = 95;
// const PAD_NEXT_CC: u8 = 94;
// const PAD_PREV_CC: u8 = 93;
const PAD_COLOR_STEP_OFF: u8 = 112;
const PAD_COLOR_STEP_SET: u8 = 53;
const PAD_COLOR_STEP_SET_OTHER_NOTE: [u8; 4] = [19, 22, 17, 16];
const PAD_COLOR_STEP_SET_AND_ACTIVE: u8 = 78;
const PAD_COLOR_STEP_ACTIVE: u8 = 3;
const PAD_COLOR_KEY: u8 = 12;
const PAD_COLOR_KEY_ACTIVE: u8 = 9;
const MIN_OCTAVE: u8 = 1;
const MAX_OCTAVE: u8 = 8;

type SelectedNotes = HashSet<Note>;

pub struct Pattern {
    instrument: usize,
    pattern: usize,
    selected_notes: SelectedNotes,
    octave: u8,
}

impl Pattern {
    pub fn new(instrument: usize, pattern: usize) -> Pattern {
        Pattern {
            selected_notes: SelectedNotes::new(),
            pattern: pattern,
            instrument: instrument,
            octave: 5,
        }
    }

    fn refresh_step(&mut self, step: Step, context: &mut UIContext) {
        let note = PAD_BAR_NOTES[step as usize];
        let channel = if step == 0 || step == BAR_SIZE - 1 {
            3
        } else {
            1
        };
        if step == context.sequencer.get_active_step() {
            if context
                .sequencer
                .get_session()
                .get_instrument(self.instrument)
                .get_pattern(self.pattern)
                .unwrap()
                .has_step_set(step)
            {
                context
                    .pad
                    .play_note(channel, note, PAD_COLOR_STEP_SET_AND_ACTIVE, 0.0);
            } else {
                context
                    .pad
                    .play_note(channel, note, PAD_COLOR_STEP_ACTIVE, 0.0);
            }
        } else {
            if context
                .sequencer
                .get_session()
                .get_instrument(self.instrument)
                .get_pattern(self.pattern)
                .unwrap()
                .has_step_set(step)
            {
                let mut any_missing = self.selected_notes.len() == 0;
                for note in self.selected_notes.clone() {
                    if !context
                        .sequencer
                        .get_session()
                        .get_instrument(self.instrument)
                        .get_pattern(self.pattern)
                        .unwrap()
                        .get_step(step)
                        .contains_key(&note)
                    {
                        any_missing = true;
                        break;
                    }
                }
                if !any_missing {
                    context
                        .pad
                        .play_note(channel, note, PAD_COLOR_STEP_SET, 0.0);
                } else {
                    context.pad.play_note(
                        channel,
                        note,
                        PAD_COLOR_STEP_SET_OTHER_NOTE[cmp::min(
                            cmp::max(
                                context
                                    .sequencer
                                    .get_session()
                                    .get_instrument(self.instrument)
                                    .get_pattern(self.pattern)
                                    .unwrap()
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
                context
                    .pad
                    .play_note(channel, note, PAD_COLOR_STEP_OFF, 0.0);
            }
        }
    }
}

impl Screen for Pattern {
    fn prepare_step(&mut self, context: &mut UIContext) {
        // clear highlighted notes
        for note in PAD_KEY_NOTES {
            let color = if note == PAD_KEY_NOTES[1] && self.octave == 5 {
                13
            } else {
                PAD_COLOR_KEY
            };
            context.pad.play_note(1, note, color, 0.0);
        }

        if (self.octave < MAX_OCTAVE) {
            context.pad.send_cc(1, PAD_NEXT_OCTAVE, 55);
        } else {
            context.pad.send_cc(1, PAD_NEXT_OCTAVE, 0);
        }
        if (self.octave > MIN_OCTAVE) {
            context.pad.send_cc(1, PAD_PREV_OCTAVE, 55);
        } else {
            context.pad.send_cc(1, PAD_PREV_OCTAVE, 0);
        }
    }

    fn handle_pad_events(&mut self, context: &mut UIContext) -> ScreenEvent {
        while context.pad.has_events() {
            let event = context.pad.pop_event().unwrap();
            let message = event.message;
            let note = message.note;
            match message.r#type {
                MidiMessageType::ControlChange => {
                    if message.velocity > 0 {
                        match message.note {
                            PAD_SESSION_CC => {
                                return ScreenEvent::SwitchToSession;
                            }
                            // PAD_NEXT_CC => {
                            //     self.instrument =
                            //         cmp::min(NUMBER_OF_INSTRUMENTS, self.instrument + 1);
                            // }
                            // PAD_PREV_CC => {
                            //     self.instrument = cmp::max(0, self.instrument - 1);
                            // }
                            _ => {}
                        }
                    }
                    println!("{} is new active instrument", self.instrument);
                }
                _ => {
                    if PAD_KEY_NOTES.contains(&note) {
                        let key_note = self.octave * 12 - 1
                            + PAD_KEY_NOTES.iter().position(|&x| x == note).unwrap() as Note;
                        match message.r#type {
                            MidiMessageType::NoteOn => {
                                context.sequencer.get_instrument(self.instrument).play_note(
                                    1,
                                    key_note,
                                    message.velocity,
                                    0.0,
                                );
                                context.pad.play_note(1, note, PAD_COLOR_KEY_ACTIVE, 0.0);
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
                                context
                                    .sequencer
                                    .get_instrument(self.instrument)
                                    .stop_note(1, key_note);
                                context.pad.play_note(1, note, PAD_COLOR_KEY, 0.0);
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
                                context
                                    .sequencer
                                    .get_session_mut()
                                    .get_instrument_mut(self.instrument)
                                    .get_pattern_mut(self.pattern)
                                    .unwrap()
                                    .clear_step(step);
                            } else {
                                let mut step_notes = if context
                                    .sequencer
                                    .get_session()
                                    .get_instrument(self.instrument)
                                    .get_pattern(self.pattern)
                                    .unwrap()
                                    .has_step_set(step)
                                {
                                    context
                                        .sequencer
                                        .get_session()
                                        .get_instrument(self.instrument)
                                        .get_pattern(self.pattern)
                                        .unwrap()
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
                                context
                                    .sequencer
                                    .get_session_mut()
                                    .get_instrument_mut(self.instrument)
                                    .get_pattern_mut(self.pattern)
                                    .unwrap()
                                    .set_step(step, &step_notes);
                            }
                            context.sequencer.save_session();
                        }
                    } else if note == PAD_NEXT_OCTAVE
                        && self.octave < MAX_OCTAVE
                        && message.velocity > 0
                    {
                        self.octave = self.octave + 1;
                    } else if note == PAD_PREV_OCTAVE
                        && self.octave > MIN_OCTAVE
                        && message.velocity > 0
                    {
                        self.octave = self.octave - 1;
                    }
                }
            }
        }
        context.pad.send_events();
        return ScreenEvent::None;
    }

    fn refresh(&mut self, context: &mut UIContext) {
        context.pad.send_cc(1, PAD_SESSION_CC, 41);
        for n in 0..BAR_SIZE {
            self.refresh_step(n, context);
        }
    }

    fn on_played_note(&mut self, context: &mut UIContext, instrument: usize, note: Note) {
        if instrument == self.instrument
            && note >= (self.octave * 12 - 1)
            && note < (self.octave + 1) * 12
        {
            let key_note = PAD_KEY_NOTES[(note - (self.octave * 12 - 1)) as usize];
            context
                .pad
                .play_note(1, key_note, PAD_COLOR_KEY_ACTIVE, 0.0);
        }
    }

    fn clear(&mut self, context: &mut UIContext) {
        for note in PAD_KEY_NOTES {
            context.pad.play_note(1, note, 0, 0.0);
        }
        for note in PAD_BAR_NOTES {
            context.pad.play_note(1, note, 0, 0.0);
        }
        context.pad.send_cc(1, PAD_NEXT_OCTAVE, 0);
        context.pad.send_cc(1, PAD_PREV_OCTAVE, 0);
        context.pad.send_cc(1, PAD_SESSION_CC, 0);
    }
}
