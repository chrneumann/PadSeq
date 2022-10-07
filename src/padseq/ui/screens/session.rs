use super::pattern::PAD_BAR_NOTES;

use crate::padseq::midi::MidiMessageType;
use crate::padseq::session::{Step, BAR_SIZE};
use crate::padseq::ui::{Screen, ScreenEvent, UIContext};

// const PAD_COLOR_PATTERN_UNSET: u8 = 103;
const PAD_COLOR_PATTERN_INACTIVE: u8 = 71;
const PAD_COLOR_PATTERN_ACTIVE: u8 = 90;
const PAD_COPY_BUTTON_NOTE: u8 = 17;
const PAD_EDIT_BUTTON_NOTE: u8 = 18;

enum Mode {
    Default,
    Edit,
    Copy,
}

pub struct Session {
    mode: Mode,
    copy_source_pattern: Option<(usize, usize)>,
}

impl Session {
    pub fn new() -> Session {
        Session {
            mode: Mode::Default,
            copy_source_pattern: None,
        }
    }

    fn refresh_step(&mut self, step: Step, context: &mut UIContext) {
        let instrument = step as usize % 8;
        let pattern = (step as usize - instrument) / 8;
        let note = PAD_BAR_NOTES[step as usize];
        let is_active = context
            .sequencer
            .get_session()
            .get_instrument(instrument)
            .get_active_pattern()
            == Some(pattern);

        let channel = 1;

        // let channel = if step == 0 || step == BAR_SIZE - 1 {
        //     3
        // } else {
        //     1
        // };

        let color = if is_active {
            PAD_COLOR_PATTERN_ACTIVE
        } else {
            PAD_COLOR_PATTERN_INACTIVE
        };
        context.pad.play_note(channel, note, color, 0.0);
    }
}

impl Screen for Session {
    fn handle_pad_events(&mut self, context: &mut UIContext) -> ScreenEvent {
        while context.pad.has_events() {
            let event = context.pad.pop_event().unwrap();
            let message = event.message;
            let note = message.note;
            match message.r#type {
                MidiMessageType::ControlChange => {
                    // if message.velocity > 0 {
                    //     match message.note {
                    //         PAD_NEXT_CC => {
                    //             self.instrument =
                    //                 cmp::min(NUMBER_OF_INSTRUMENTS, self.instrument + 1);
                    //         }
                    //         PAD_PREV_CC => {
                    //             self.instrument = cmp::max(0, self.instrument - 1);
                    //         }
                    //         _ => {}
                    //     }
                    // }
                    // println!("{} is new active instrument", self.instrument);
                }
                _ => {
                    if PAD_BAR_NOTES.contains(&note) {
                        if message.velocity > 0 {
                            let step =
                                PAD_BAR_NOTES.iter().position(|&x| x == note).unwrap() as usize;
                            let instrument = step % 8;
                            let pattern = (step - instrument) / 8;
                            println!(
                                "step {}, instrument {}, pattern {}",
                                step, instrument, pattern
                            );

                            if matches!(&self.mode, Mode::Edit) {
                                return ScreenEvent::SwitchToPattern(instrument, pattern);
                            }

                            if matches!(&self.mode, Mode::Copy) {
                                if matches!(&self.copy_source_pattern, None) {
                                    self.copy_source_pattern = Some((instrument, pattern));
                                } else {
                                    let (src_instrument, src_pattern) =
                                        self.copy_source_pattern.unwrap();
                                    let the_pattern = context
                                        .sequencer
                                        .get_session()
                                        .get_instrument(src_instrument)
                                        .get_pattern(src_pattern)
                                        .unwrap()
                                        .clone();
                                    context
                                        .sequencer
                                        .get_session_mut()
                                        .get_instrument_mut(instrument)
                                        .set_pattern(pattern, &the_pattern);
                                    self.copy_source_pattern = None;
                                    self.mode = Mode::Default;
                                }
                                continue;
                            }

                            if context
                                .sequencer
                                .get_session()
                                .get_instrument(instrument)
                                .get_active_pattern()
                                == Some(pattern)
                            {
                                println!("set none");
                                context
                                    .sequencer
                                    .get_session_mut()
                                    .get_instrument_mut(instrument)
                                    .set_active_pattern(None);
                            } else {
                                println!("set active");
                                context
                                    .sequencer
                                    .get_session_mut()
                                    .get_instrument_mut(instrument)
                                    .set_active_pattern(Some(pattern));
                            }
                            context.sequencer.save_session();
                        }
                    } else if note == PAD_EDIT_BUTTON_NOTE {
                        if message.velocity > 0 {
                            self.mode = match self.mode {
                                Mode::Edit => Mode::Default,
                                _ => Mode::Edit,
                            };
                        }
                    } else if note == PAD_COPY_BUTTON_NOTE {
                        if message.velocity > 0 {
                            self.mode = match self.mode {
                                Mode::Copy => Mode::Default,
                                _ => Mode::Copy,
                            };
                        }
                    }
                }
            }
        }
        context.pad.send_events();
        return ScreenEvent::None;
    }

    fn refresh(&mut self, context: &mut UIContext) {
        for n in 0..BAR_SIZE {
            self.refresh_step(n, context);
        }
        context.pad.play_note(
            if matches!(&self.mode, Mode::Edit) {
                3
            } else {
                1
            },
            PAD_EDIT_BUTTON_NOTE,
            5,
            0.0,
        );
        context.pad.play_note(
            if matches!(&self.mode, Mode::Copy) {
                3
            } else {
                1
            },
            PAD_COPY_BUTTON_NOTE,
            124,
            0.0,
        );
    }

    fn clear(&mut self, context: &mut UIContext) {
        for note in PAD_BAR_NOTES {
            context.pad.play_note(1, note, 0, 0.0);
        }
        for note in [PAD_COPY_BUTTON_NOTE, PAD_EDIT_BUTTON_NOTE] {
            context.pad.play_note(1, note, 0, 0.0);
        }
    }
}
