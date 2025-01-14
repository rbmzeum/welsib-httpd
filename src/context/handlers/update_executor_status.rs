use super::super::WelsibContext;
// use super::super::WelsibState;
use std::time::Duration;
use std::thread::sleep;

impl WelsibContext {
    pub fn do_update_executor_status(&mut self) {
        loop {
            // println!("Begin update status");
            match self.sender().lock().as_deref_mut() {
                Ok(sender) => match sender.send(true) {
                    Ok(()) => {
                        // println!("Notify update status");
                    }
                    Err(e) => {
                        eprintln!("Error {:#?}", e);
                    }
                },
                Err(e) => {
                    eprintln!("Error {:#?}", e);
                }
            };
            sleep(Duration::from_secs(4));
        }
    }
}