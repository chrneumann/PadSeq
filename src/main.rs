use std::env;
use std::process;
mod padseq;
use padseq::sequencer::Sequencer;
use padseq::ui::UI;

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path = match args.len() {
        1 => None,
        2 => Some(args[1].clone()),
        _ => {
            println!("Expects only one optional argument, the path to a session.json file");
            process::exit(1);
        }
    };
    let mut ui = UI::new(Sequencer::new(file_path));
    print!("Connected");
    ui.run();
}
