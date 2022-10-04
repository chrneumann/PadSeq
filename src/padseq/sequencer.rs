use super::midi::Instrument;
use super::session::{Note, Session, Step, BAR_SIZE};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

pub const NUMBER_OF_INSTRUMENTS: usize = 8;

type StepSize = f64;
const BASE_BPM: StepSize = 126.0;
const STEP_LENGTH: StepSize = 1000.0 * 60.0 / (4.0 * BASE_BPM);

pub enum WaitResult {
    Step,
    Intermediate,
}

pub type PlayedNotes = Vec<(usize, Note)>;

pub struct Sequencer {
    session: Session,
    instruments: Vec<Instrument>,
    session_file_path: Option<String>,
    active_step: Step,
    last_step: Instant,
}

impl Sequencer {
    pub fn new(file_path: Option<String>) -> Sequencer {
        let session = match file_path {
            Some(ref path) => match Path::new(path).exists() {
                true => Session::from_json(
                    &fs::read_to_string(path).expect("Should have been able to read the file"),
                )
                .unwrap(),
                false => Session::new(NUMBER_OF_INSTRUMENTS),
            },
            None => Session::new(NUMBER_OF_INSTRUMENTS),
        };
        Sequencer {
            session,
            session_file_path: file_path.clone(),
            instruments: Vec::new(),
            active_step: 0,
            last_step: Instant::now(),
        }
    }

    pub fn connect(&mut self) {
        for n in 0..NUMBER_OF_INSTRUMENTS {
            let name: String = format!("instrument {}", n);
            let mut instrument = Instrument::new(&name);
            instrument.set_debug(true);
            instrument.connect_out(0);
            self.instruments.push(instrument);
        }
        print!("Connect done");
    }

    fn play_notes(&mut self) -> PlayedNotes {
        let mut played_notes = PlayedNotes::new();
        for instrument in 0..NUMBER_OF_INSTRUMENTS {
            if self
                .session
                .get_instrument(instrument)
                .get_pattern(0)
                .has_step_set(self.active_step)
            {
                let notes = self
                    .session
                    .get_instrument(instrument)
                    .get_pattern(0)
                    .get_step(self.active_step);
                for (note, velocity) in notes {
                    println!("play {}", note);
                    self.instruments[instrument].play_note(1, *note, *velocity, STEP_LENGTH); // TODO
                    played_notes.push((instrument, *note));
                }
            }
        }
        return played_notes;
    }

    pub fn save_session(&self) {
        match &self.session_file_path {
            Some(path) => {
                let data = self.session.to_json().unwrap();
                println!("{} {}", path, data);
                fs::write(path, data).expect("Unable to write file");
            }
            None => {}
        };
    }

    pub fn get_session(&mut self) -> &mut Session {
        return &mut self.session;
    }

    pub fn get_active_step(&self) -> Step {
        return self.active_step;
    }

    pub fn process_step(&mut self) -> PlayedNotes {
        self.active_step = (self.active_step + 1) % BAR_SIZE;
        return self.play_notes();
    }

    pub fn wait(&mut self) -> WaitResult {
        if self.last_step.elapsed().as_micros() >= (STEP_LENGTH * 1000.0).floor() as u128 {
            self.last_step = Instant::now();
            return WaitResult::Step;
        }
        for n in 0..NUMBER_OF_INSTRUMENTS {
            self.instruments[n].send_events();
        }
        return WaitResult::Intermediate;
    }

    pub fn get_instrument(&mut self, index: usize) -> &mut Instrument {
        return &mut self.instruments[index];
    }
}
