use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;

struct CheckInCounter {
    id: u32,
    baggage_log: Arc<Mutex<Vec<String>>>,
}

impl CheckInCounter {
    fn new(id: u32, baggage_log: Arc<Mutex<Vec<String>>>) -> Self {
        Self { id, baggage_log }
    }

    fn process_baggage(&self, baggage_id: &str) {
        let mut log = self.baggage_log.lock().unwrap();

        log.push(format!(
            "Check-in {} registrerede bagage: {}",
            self.id, baggage_id
        ));

        println!("Skranke {} registrerede bagage: {}", self.id, baggage_id);
    }
}
struct PassengerSummoner {
    id: u32,
    passenger_list: Arc<Mutex<Vec<String>>>,
}

impl PassengerSummoner {
    fn new(id: u32, passenger_list:Arc<Mutex<Vec<String>>>) -> Self {
        Self { id, passenger_list}
    }
    fn start_summon(&self){
        let passenger_list = Arc::clone(&self.passenger_list);
        let summoner_id = self.id;

        thread::spawn(move || {
            let mut passenger_counter = 0;
            loop {
                thread::sleep(Duration::from_secs(2));
                
                passenger_counter += 1;
                let passenger_id = format!("PASS-{}-{}", summoner_id, passenger_counter);
                
                let mut list = passenger_list.lock().unwrap();
                list.push(passenger_id.clone());
                println!("Summoner {}: Created passenger {}", summoner_id, passenger_id);
            }
        });
    }
}

fn main() {
    let passenger_list: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let baggage_log = Arc::new(Mutex::new(Vec::new()));

    let summoner1 = PassengerSummoner::new(1, passenger_list.clone());
    let counter1 = CheckInCounter::new(1, baggage_log.clone());
    summoner1.start_summon();
    let handle = thread::spawn(move || {
        counter1.process_baggage("BAG123");
    });
    handle.join().unwrap();
}