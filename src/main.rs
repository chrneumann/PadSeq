mod padseq;
use padseq::sequencer::Sequencer;

fn main() {
    let mut seq = Sequencer::new();
    seq.connect();
    print!("Connected");
    seq.run();
}
