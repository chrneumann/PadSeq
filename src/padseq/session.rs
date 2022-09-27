use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::collections::HashMap;

pub type Step = u8;
pub type Channel = u8;
pub type Note = u8;
pub type Velocity = u8;
pub const BAR_SIZE: Step = 32;
pub type StepNotes = HashMap<Note, Velocity>;
pub type Bar = HashMap<Step, StepNotes>;

#[derive(Serialize, Deserialize)]
pub struct Pattern {
    bar: Bar,
}

impl Pattern {
    pub fn new() -> Pattern {
        Pattern { bar: Bar::new() }
    }

    pub fn set_step(&mut self, step: Step, notes: &StepNotes) {
        self.bar.insert(step, notes.clone());
    }

    pub fn get_step(&self, step: Step) -> &StepNotes {
        // println!("{:?} {} -> {}", self.bar, step, self.bar[step as usize]);
        return self.bar.get(&step).unwrap();
    }

    pub fn clear_step(&mut self, step: Step) {
        self.bar.remove(&step);
    }

    pub fn has_step_set(&self, step: Step) -> bool {
        return self.bar.contains_key(&step);
    }
}

#[derive(Serialize, Deserialize)]
pub struct Instrument {
    patterns: Vec<Pattern>,
}

impl Instrument {
    pub fn new() -> Instrument {
        let mut patterns = Vec::new();
        patterns.push(Pattern::new());
        Instrument { patterns: patterns }
    }

    pub fn get_pattern(&mut self, index: usize) -> &mut Pattern {
        &mut self.patterns[index]
    }
}

#[derive(Serialize, Deserialize)]
pub struct Session {
    instruments: Vec<Instrument>,
}

impl Session {
    pub fn new(number_of_instruments: usize) -> Session {
        let mut instruments = Vec::new();
        for _ in 0..number_of_instruments {
            instruments.push(Instrument::new());
        }
        return Session {
            instruments: instruments,
        };
    }

    pub fn from_json(json: &str) -> Result<Session> {
        let s: Session = serde_json::from_str(json)?;
        Ok(s)
    }

    pub fn get_instrument(&mut self, index: usize) -> &mut Instrument {
        &mut self.instruments[index]
    }

    pub fn to_json(&self) -> Result<String> {
        let j = serde_json::to_string(&self)?;
        Ok(j)
    }
}
