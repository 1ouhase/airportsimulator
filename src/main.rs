use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;
use chrono::Local;
use std::io::{self, BufRead};

struct Passenger {
    number: u32,
    flight: String,
}

struct Baggage {
    id: String,
    flight: String,
}

struct Sorter;

impl Sorter {
    fn start(belt: Arc<Mutex<Vec<Baggage>>>, sorted_belt: Arc<Mutex<Vec<Baggage>>>){
        thread::spawn(move || {
            loop {
                let baggage = {
                    let mut b = belt.lock().unwrap();
                    b.pop()
                };

                match baggage {
                    Some(bag) => {
                        println!("[Sorter] sorting bag {} for flight {}", bag.id, bag.flight);
                        thread::sleep(Duration::from_secs(1));
                        println!("[Sorter] Bag {} sorted at {}", bag.id, Local::now().format("%H:%M%S"));
                        sorted_belt.lock().unwrap().push(bag);
                    }
                    None => { thread::sleep(Duration::from_millis(500));}
                }
            }

        });
    }
}

struct Counter {
    id: u32,
    is_open: bool,
}

impl Counter {
    fn new(id: u32) -> Self{
        Self { id, is_open: false}
    }

    fn open(&mut self) {
        self.is_open = true;
        println!("[System] counter {} is open", self.id);
    }
    
    fn close(&mut self) {
        self.is_open = false;
        println!("[System] counter {} is now closed", self.id);
    }

    fn start(id: u32,
        queue: Arc<Mutex<Vec<Passenger>>>, 
        belt: Arc<Mutex<Vec<Baggage>>>,
        counter: Arc<Mutex<Vec<Counter>>>,
        bag_counter: Arc<Mutex<u32>>) {

        thread::spawn(move || {
            loop {
                let is_open = {
                    let counters = counters.lock().unwrap();
                    counters.iter().find(|c| c.id == id).unwrap().is_open
                };

                if is_open {
                    let passenger = {
                        let mut q = queue.lock().unwrap();
                        q.pop()
                    };
                    match passenger {
                        Some(p) => {
                            println!("[Counter {}] Checking in passenger {} for flight {}", id, p.number, p.flight);
                            thread::sleep(Duration::from_secs(3));

                            let bag_id = {
                                let mut bc = bag_counter.lock().unwrap();
                                *bc += 1;
                                format!("BAG-{:03}", bc)
                            };
                            let baggage = Baggage { id: bag_id.clone(), flight: p.flight.clone() };
                            belt.lock().unwrap().push(baggage);
                            println!("[Counter {}] Bag {} for flight {} placed on belt", id, bag_id, p.flight);
                        }
                        None => { thread::sleep(Duration::from_millis(500));}
                    }
                } else {
                    thread::sleep(Duration::from_millis(500));
                }
            }
        });
    }
}

struct Gate {
    id: u32,
    flight: String,
    is_open: bool,
}

impl Gate {
    fn new(id: u32, flight: &str) -> Self {
        Self { id, flight: String::from(flight), is_open: false }
    }

    fn open(&mut self) {
        self.is_open = true;
        println!("[System] Gate {} is now open", self.id);
    }

    fn close(&mut self) {
        self.is_open = false;
        println!("[System] Gate {} is now closed", self.id);
    }

    fn start(id: u32, flight: String, sorted_belt: Arc<Mutex<Vec<Baggage>>>, gates: Arc<Mutex<Vec<Gate>>>) {
        thread::spawn(move || {
            loop {
                let is_open = {
                    let gates = gates.lock().unwrap();
                    gates.iter().find(|g| g.id == id).unwrap().is_open
                };

                if is_open {
                    let bag = {
                        let mut belt = sorted_belt.lock().unwrap();
                        let pos = belt.iter().position(|b| b.flight == flight);
                        pos.map(|i| belt.remove(i))
                    };

                    match bag {
                        Some(b) => {
                            println!("[Gate {}] Received bag {} for flight {} at {}", id, b.id, b.flight, Local::now().format("%H:%M:%S"));
                        }
                        None => { thread::sleep(Duration::from_millis(500)); }
                    }
                } else {
                    thread::sleep(Duration::from_millis(500));
                }
            }
        });
    }
}

fn main() {
    let flight = ["SAS001", "SAS002", "SAS003"];
    let queue: Arc<Mutex<Vec<Passenger>>> = Arc::new(Mutex::new(Vec::new()));
    let belt: Arc<Mutex<Vec<Baggage>>> = Arc::new(Mutex::new(Vec::new()));
    let sorted_belt: Arc<Mutex<Vec<Baggage>>> = Arc::new(Mutex::new(Vec::new()));

    let gates: Arc<Mutex<Vec<Gate>>> = Arc::new(Mutex::new(vec![
            Gate::new(1, "SAS001"),
            Gate::new(2, "SAS002"),
            Gate::new(3, "SAS003"),
    ]));

    let counter: Arc<Mutex<Vec<Counter>>> = Arc::new(Mutex::new(vec![
            Counter::new(1),
            Counter::new(2),
    ]));

    let bag_counter = Arc::new(Mutex::new(0u32));

    let queue_for_sum = Arc::clone(&queue);
    thread::spawn(move || {
        let mut count = 0;
        loop {
            thread::sleep(Duration::from_secs(3));
            count += 1;
            let flight = flight[count as usize % flight.len()];
            let passenger = Passenger { number: count, flight: String::from(flight) };
            let mut q = queue_for_sum.lock().unwrap();
            println!("[Summoner] passenger {} arrived for flight {}", passenger.number, passenger.flight);
            q.push(passenger);
        }
    });

    for counter_id in 1..=2 {
        Counter::start(
            counter_id,
            Arc::clone(&queue),
            Arc::clone(&belt),
            Arc::clone(&counters), 
            Arc::clone(&bag_counter),
            );
    }

    Sorter::start(Arc::clone(&belt), Arc::clone(&sorted_belt));

    let gate_flights = vec![
        (1, "SAS001"), (2, "SAS002"), (3, "SAS003")
    ];
    for (gate_id, flight) in gate_flights {
        Gate::start(
            gate_id,
            String::from(flight),
            Arc::clone(&sorted_belt),
            Arc::clone(&gates),
        );
    }

    // user input to open start threads go here
    //
    //
}
















