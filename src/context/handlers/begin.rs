use super::super::WelsibContext;
use super::super::WelsibState;

impl WelsibContext {
    pub fn do_begin(&mut self) {
        println!("\nBegin");
        let next_state = if self.stream().is_some() {
            WelsibState::AwaitReadRequest
        } else {
            WelsibState::AwaitUpdateExecutorStatus
        };

        self.set_state(next_state);
    }
}