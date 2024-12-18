use std::time::{Duration, SystemTime};
use std::thread::sleep;
use std::{io, usize};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::thread;
use serde::{Serialize, Deserialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
struct R {
    button: i32,
}

enum TimerState {
    Ready,
    Started,
    Paused,
}

struct Timer {
    state: TimerState,
    last_update: SystemTime,
    duration: Duration,
    cur_interval: usize,
    intervals: Vec<Duration>,
}

impl Timer {
    fn new() -> Self {
        let intervals = vec![
            Duration::from_secs(15),
            Duration::from_secs(5),
            Duration::from_secs(15),
            Duration::from_secs(5),
            Duration::from_secs(15),
            Duration::from_secs(15),
        ];

        return Self {
            state: TimerState::Ready,
            last_update: SystemTime::now(),
            duration: intervals[0],
            cur_interval: 0,
            intervals,
        }
    }

    fn start(&mut self) {
        if let TimerState::Ready = self.state {
            self.state = TimerState::Started;
            self.duration = self.intervals[self.cur_interval];
        }
    }

    fn next(&mut self) {
        if self.cur_interval >= self.intervals.len() - 1 {
            self.cur_interval = 0;
        } else {
            self.cur_interval += 1; 
        }
        self.state = TimerState::Ready;
        self.duration = self.intervals[self.cur_interval];
    }
    
    fn toggle_pause(&mut self) {
        if let TimerState::Started = self.state {
            self.state = TimerState::Paused;
        } else if let TimerState::Paused = self.state {
            self.state = TimerState::Started;
        } 
    }

    fn update(&mut self) {
        let now = SystemTime::now();
        match self.state {
            TimerState::Started => {
                let since = now.duration_since(self.last_update).unwrap();
                if since >= self.duration {
                    self.next();
                } else {
                    self.duration -= since;
                }
            },
            _ => { }
        };

        self.last_update = now;
    }

    fn display(&self) -> String {
        let min = self.duration.as_secs() / 60;
        let sec = self.duration.as_secs() % 60;
        format!("{:02}:{:02}", min, sec)
    }
}

fn main() -> io::Result<()>{
    let stdin_channel = spawn_stdin_channel();
    let mut timer = Timer::new();

    loop {
        timer.update();
        match stdin_channel.try_recv() {
            Ok(key) => {
                let r: R = serde_json::from_str(&key).unwrap();
                if r.button == 1 {
                    timer.start()
                } else if r.button == 3 {
                    timer.toggle_pause();
                }
            }
            Err(TryRecvError::Empty) => {},
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        };
        let msg = json!({"full_text": format!("<span>{}</span>", timer.display())});
        let serialized = serde_json::to_string(&msg).unwrap();
        println!("{}", serialized);
        sleep(Duration::from_secs(1));
    }
}

fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel::<String>();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        tx.send(buffer).unwrap();
    });
    rx
}
