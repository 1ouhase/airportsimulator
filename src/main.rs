use std::sync::{Arc, Mutex};
use std::thread;

struct CheckInCounter {
    id: u32,
    baggage_log: Arc<Mutex<Vec<String>>>,
}

impl CheckInCounter {
    fn new(id: u32, baggage_log: Arc<Mutex<Vec<String>>>) -> Self {
        Self { id, baggage_log }
    }

    fn process_baggage(&self, baggage_id: &str, passenger_list: &Arc<Mutex<Vec<String>>>) {
        let mut list = passenger_list.lock().unwrap();
        let mut log = self.baggage_log.lock().unwrap();
        log.push(format!(
            "Check-in {} registrerede bagage: {}",
            self.id, baggage_id
        ));
        println!("Skranke {} registrerede bagage: {}", self.id, baggage_id);
    }
}
struct Passenger {}
fn main() {
    let passenger_list: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let baggage_log = Arc::new(Mutex::new(Vec::new()));
    let counter1 = CheckInCounter::new(1, baggage_log.clone());
    let handle = thread::spawn(move || {
        counter1.process_baggage("BAG123", &passenger_list);
    });
    handle.join().unwrap();
}