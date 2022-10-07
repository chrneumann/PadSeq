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

#[derive(Serialize, Deserialize, Clone)]
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
    patterns: HashMap<usize, Pattern>,
    active_pattern: Option<usize>,
}

impl Instrument {
    pub fn new() -> Instrument {
        Instrument {
            patterns: HashMap::new(),
            active_pattern: None,
        }
    }

    pub fn has_pattern(&self, index: usize) -> bool {
        return self.patterns.contains_key(&index);
    }

    pub fn get_pattern(&self, index: usize) -> Option<&Pattern> {
        return self.patterns.get(&index);
    }

    pub fn get_pattern_mut(&mut self, index: usize) -> Option<&mut Pattern> {
        return self.patterns.get_mut(&index);
    }

    pub fn set_pattern(&mut self, index: usize, pattern: &Pattern) {
        self.patterns.insert(index, pattern.clone());
    }

    pub fn get_active_pattern(&self) -> Option<usize> {
        return self.active_pattern;
    }

    pub fn set_active_pattern(&mut self, pattern: Option<usize>) {
        self.active_pattern = pattern;
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

    pub fn get_instrument(&self, index: usize) -> &Instrument {
        &self.instruments[index]
    }

    pub fn get_instrument_mut(&mut self, index: usize) -> &mut Instrument {
        &mut self.instruments[index]
    }

    pub fn to_json(&self) -> Result<String> {
        let j = serde_json::to_string(&self)?;
        Ok(j)
    }
}
