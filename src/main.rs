use chrono::Local;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};
//airport stuff
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
    fn start(
        belt: Arc<Mutex<Vec<Baggage>>>,
        sorted_belt: Arc<Mutex<Vec<Baggage>>>,
        log: Arc<Mutex<Vec<String>>>,
    ) {
        thread::spawn(move || {
            loop {
                let baggage = {
                    let mut b = belt.lock().unwrap();
                    b.pop()
                };

                match baggage {
                    Some(bag) => {
                        log.lock().unwrap().push(format!("[Sorter] sorting bag {} for flight {}", bag.id, bag.flight));
                        thread::sleep(Duration::from_secs(1));
                        log.lock().unwrap().push(format!("[Sorter] bag {} sorted at {}", bag.id, Local::now().format("%H:%M:%S")));
                        sorted_belt.lock().unwrap().push(bag);
                    }
                    None => {
                        thread::sleep(Duration::from_millis(500));
                    }
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
    fn new(id: u32) -> Self {
        Self { id, is_open: false }
    }

    fn open(&mut self) {
        self.is_open = true;
    }

    fn close(&mut self) {
        self.is_open = false;
    }

    fn start(
        id: u32,
        queue: Arc<Mutex<Vec<Passenger>>>,
        belt: Arc<Mutex<Vec<Baggage>>>,
        counters: Arc<Mutex<Vec<Counter>>>,
        bag_counter: Arc<Mutex<u32>>,
        log: Arc<Mutex<Vec<String>>>,
    ) {
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
                            log.lock().unwrap().push(format!("[Counter {}] Checking in passenger {} for flight {}", id, p.number, p.flight));
                            thread::sleep(Duration::from_secs(3));

                            let bag_id = {
                                let mut bc = bag_counter.lock().unwrap();
                                *bc += 1;
                                format!("BAG-{:03}", bc)
                            };
                            let baggage = Baggage {
                                id: bag_id.clone(),
                                flight: p.flight.clone(),
                            };
                            belt.lock().unwrap().push(baggage);
                            log.lock().unwrap().push(format!("[Counter {}] Bag {} for flight {} placed on belt", id, bag_id, p.flight));
                        }
                        None => {
                            thread::sleep(Duration::from_millis(500));
                        }
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
        Self {
            id,
            flight: String::from(flight),
            is_open: false,
        }
    }

    fn open(&mut self) {
        self.is_open = true;
    }

    fn close(&mut self) {
        self.is_open = false;
    }

    fn start(
        id: u32,
        flight: String,
        sorted_belt: Arc<Mutex<Vec<Baggage>>>,
        gates: Arc<Mutex<Vec<Gate>>>,
        log: Arc<Mutex<Vec<String>>>,
    ) {
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
                            log.lock().unwrap().push(format!("[Gate {}] Recived bag {} for flight {} at {}",
                                    id, b.id, b.flight, Local::now().format("%H:%M:%S")));
                        }
                        None => {
                            thread::sleep(Duration::from_millis(500));
                        }
                    }
                } else {
                    thread::sleep(Duration::from_millis(500));
                }
            }
        });
    }
}

// app struct, includes every variables the program needs to run
struct App {
    exit: bool,
    input: String,
    counters: Arc<Mutex<Vec<Counter>>>,
    gates: Arc<Mutex<Vec<Gate>>>,
    queue: Arc<Mutex<Vec<Passenger>>>,
    log: Arc<Mutex<Vec<String>>>,
    system_log: Arc<Mutex<Vec<String>>>,
}

impl App {
    fn new(
        counters: Arc<Mutex<Vec<Counter>>>,
        gates: Arc<Mutex<Vec<Gate>>>,
        queue: Arc<Mutex<Vec<Passenger>>>,
        log: Arc<Mutex<Vec<String>>>,
        system_log: Arc<Mutex<Vec<String>>>,
    ) -> Self {
        Self { exit: false, input: String::new(), counters, gates, queue, log, system_log }
    }

    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        // Split screen into top, middle, system log, and command bar
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6),  // top: counters + gates
                Constraint::Min(5),     // middle: queue + baggage log
                Constraint::Length(4),  // system messages
                Constraint::Length(3),  // command input
            ])
            .split(area);

        // Split top row into counters (left) and gates (right)
        let top_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[0]);

        // Split middle row into queue (left) and log (right)
        let mid_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);

        // --- Counters panel ---
        let counter_lines: Vec<Line> = self.counters.lock().unwrap()
            .iter()
            .map(|c| {
                if c.is_open {
                    Line::from(Span::styled(
                        format!("  Counter {}:  OPEN", c.id),
                        Color::Green,
                    ))
                } else {
                    Line::from(Span::styled(
                        format!("  Counter {}:  CLOSED", c.id),
                        Color::Red,
                    ))
                }
            })
            .collect();

        frame.render_widget(
            Paragraph::new(counter_lines).block(Block::bordered().title(" Counters ")),
            top_cols[0],
        );

        // --- Gates panel ---
        let gate_lines: Vec<Line> = self.gates.lock().unwrap()
            .iter()
            .map(|g| {
                if g.is_open {
                    Line::from(Span::styled(
                        format!("  Gate {} ({}):  OPEN", g.id, g.flight),
                        Color::Green,
                    ))
                } else {
                    Line::from(Span::styled(
                        format!("  Gate {} ({}):  CLOSED", g.id, g.flight),
                        Color::Red,
                    ))
                }
            })
            .collect();

        frame.render_widget(
            Paragraph::new(gate_lines).block(Block::bordered().title(" Gates ")),
            top_cols[1],
        );

        // --- Passenger queue panel ---
        let queue_lines: Vec<Line> = self.queue.lock().unwrap()
            .iter()
            .map(|p| Line::from(format!("  Passenger {} -> {}", p.number, p.flight)))
            .collect();

        frame.render_widget(
            Paragraph::new(queue_lines).block(Block::bordered().title(" Passenger Queue ")),
            mid_cols[0],
        );

        // --- Baggage log panel ---
        let log = self.log.lock().unwrap();
        // Show only the most recent entries that fit
        let log_lines: Vec<Line> = log.iter().rev()
            .take(mid_cols[1].height as usize)
            .map(|entry| Line::from(format!("  {}", entry)))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        frame.render_widget(
            Paragraph::new(log_lines).block(Block::bordered().title(" Baggage Log ")),
            mid_cols[1],
        );

        // --- System log panel ---
        let system_lines: Vec<Line> = self.system_log.lock().unwrap()
            .iter().rev()
            .take(rows[2].height as usize)
            .map(|entry| Line::from(format!("  {}", entry)))
            .collect::<Vec<_>>()
            .into_iter().rev()
            .collect();

        frame.render_widget(
            Paragraph::new(system_lines).block(Block::bordered().title(" System ")),
            rows[2],
        );

        // --- Command input ---
        frame.render_widget(
            Paragraph::new(format!(" > {}", self.input))
                .block(Block::bordered().title(" Command (e.g. 'open counter 1') | q to quit ")),
            rows[3],
        );
    }

    fn handle_events(&mut self) -> io::Result<()> {
        // Only block for 200ms so the screen refreshes regularly
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => self.exit = true,
                        KeyCode::Char(c)   => self.input.push(c),
                        KeyCode::Backspace => { self.input.pop(); }
                        KeyCode::Enter     => self.handle_command(),
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_command(&mut self) {
        let input = self.input.trim().to_string();
        self.input.clear();

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.len() != 3 {
            self.system_log.lock().unwrap().push(format!("Unknown command: '{}'", input));
            return;
        }

        let action = parts[0];
        let target = parts[1];
        let id: u32 = match parts[2].parse() {
            Ok(n) => n,
            Err(_) => {
                self.system_log.lock().unwrap().push("Invalid id".to_string());
                return;
            }
        };

        match target {
            "counter" => {
                let mut counters = self.counters.lock().unwrap();
                match counters.iter_mut().find(|c| c.id == id) {
                    Some(counter) => match action {
                        "open"  => { counter.open();  self.system_log.lock().unwrap().push(format!("Counter {} opened", id)); }
                        "close" => { counter.close(); self.system_log.lock().unwrap().push(format!("Counter {} closed", id)); }
                        _ => { self.system_log.lock().unwrap().push("Use 'open' or 'close'".to_string()); }
                    },
                    None => { self.system_log.lock().unwrap().push(format!("No counter with id {}", id)); }
                }
            }
            "gate" => {
                let mut gates = self.gates.lock().unwrap();
                match gates.iter_mut().find(|g| g.id == id) {
                    Some(gate) => match action {
                        "open"  => { gate.open();  self.system_log.lock().unwrap().push(format!("Gate {} opened", id)); }
                        "close" => { gate.close(); self.system_log.lock().unwrap().push(format!("Gate {} closed", id)); }
                        _ => { self.system_log.lock().unwrap().push("Use 'open' or 'close'".to_string()); }
                    },
                    None => { self.system_log.lock().unwrap().push(format!("No gate with id {}", id)); }
                }
            }
            _ => { self.system_log.lock().unwrap().push("Use 'counter' or 'gate'".to_string()); }
        }
    }
}

// program start... MAIN

fn main() -> io::Result<()> {
    let flights = ["SK101", "SK202", "SK303"];

    let queue:       Arc<Mutex<Vec<Passenger>>> = Arc::new(Mutex::new(Vec::new()));
    let belt:        Arc<Mutex<Vec<Baggage>>>   = Arc::new(Mutex::new(Vec::new()));
    let sorted_belt: Arc<Mutex<Vec<Baggage>>>   = Arc::new(Mutex::new(Vec::new()));
    let log:         Arc<Mutex<Vec<String>>>    = Arc::new(Mutex::new(Vec::new()));
    let system_log:  Arc<Mutex<Vec<String>>>    = Arc::new(Mutex::new(Vec::new()));
    let bag_counter: Arc<Mutex<u32>>            = Arc::new(Mutex::new(0));

    let counters: Arc<Mutex<Vec<Counter>>> = Arc::new(Mutex::new(vec![
        Counter::new(1),
        Counter::new(2),
        Counter::new(3)
    ]));

    let gates: Arc<Mutex<Vec<Gate>>> = Arc::new(Mutex::new(vec![
        Gate::new(1, "SK101"),
        Gate::new(2, "SK202"),
        Gate::new(3, "SK303"),
    ]));

    // Passenger summoner thread start
    let queue_for_summoner = Arc::clone(&queue);
    thread::spawn(move || {
        let mut count = 0;
        loop {
            thread::sleep(Duration::from_secs(2));
            count += 1;
            let flight = flights[count as usize % flights.len()];
            queue_for_summoner.lock().unwrap().push(Passenger {
                number: count,
                flight: String::from(flight),
            });
        }
    });

    // --- Counter threads ---
    for counter_id in 1..=2 {
        Counter::start(
            counter_id,
            Arc::clone(&queue),
            Arc::clone(&belt),
            Arc::clone(&counters),
            Arc::clone(&bag_counter),
            Arc::clone(&log),
        );
    }

    // --- Sorter thread ---
    Sorter::start(Arc::clone(&belt), Arc::clone(&sorted_belt), Arc::clone(&log));

    // --- Gate threads ---
    let gate_flights = [(1, "SK101"), (2, "SK202"), (3, "SK303")];
    for (gate_id, flight) in gate_flights {
        Gate::start(
            gate_id,
            String::from(flight),
            Arc::clone(&sorted_belt),
            Arc::clone(&gates),
            Arc::clone(&log),
        );
    }

    // --- Start TUI ---
    let mut terminal = ratatui::init();
    let result = App::new(
        Arc::clone(&counters),
        Arc::clone(&gates),
        Arc::clone(&queue),
        Arc::clone(&log),
        Arc::clone(&system_log),
    ).run(&mut terminal);
    ratatui::restore();
    result
}
