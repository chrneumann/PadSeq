use std::collections::HashMap;

pub type Step = u8;
pub type Channel = u8;
pub type Note = u8;
pub type Velocity = u8;
pub const BAR_SIZE: Step = 32;
pub type StepNotes = HashMap<Note, Velocity>;
pub type Bar = HashMap<Step, StepNotes>;

pub struct Session {
    bar: Bar,
}

impl Session {
    pub fn new() -> Session {
        Session { bar: Bar::new() }
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
