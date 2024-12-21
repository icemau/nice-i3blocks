use clap::Parser;
use std::str::FromStr;
use std::time::{Duration, SystemTime};
use std::{io, usize};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::thread::{self, sleep};
use serde::Deserialize;
use serde_json::json;


/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of intervals
    #[arg(short, long, default_value_t = 5)]
    count: usize,

    /// Duration of the focus intervals
    #[arg(short, long, default_value = "25m", value_parser = parse_duration)]
    focus: humantime::Duration,

    /// Duration of the pause intervals
    #[arg(short, long, default_value = "5m", value_parser = parse_duration)]
    pause: humantime::Duration,

    /// Duration of the long pause interval
    #[arg(short, long, default_value = "15m", value_parser = parse_duration)]
    long_pause: humantime::Duration,
}

fn parse_duration(arg: &str) -> Result<humantime::Duration, humantime::DurationError> {
    humantime::Duration::from_str(arg)
}

#[derive(Deserialize, Debug)]
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
    fn new(count: usize, focus: Duration, pause: Duration, long_pause: Duration) -> Self {
        if count == 0 {
            panic!("Timer interval count must be at least 1");
        }

        let mut intervals = Vec::with_capacity(count * 2);
        for i in 0..count {
            intervals.push(focus);
            if i == count - 1 {
                intervals.push(long_pause);
            } else {
                intervals.push(pause);
            }
        }

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
        match self.state {
            TimerState::Started => self.state = TimerState::Paused,
            TimerState::Paused => self.state = TimerState::Started,
            _ => {}
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

    fn display_interval(&self) -> String {
        format!("{}/{}", self.cur_interval / 2, self.intervals.len() / 2)
    }

    fn display_duration(&self) -> String {
        let min = self.duration.as_secs() / 60;
        let sec = self.duration.as_secs() % 60;
        format!("{:02}:{:02}", min, sec)
    }

    fn display_action(&self) -> &'static str {
        if self.cur_interval % 2 == 0 {
            "F"
        } else {
            "P"
        }
    }
}

fn main() -> io::Result<()>{
    let args = Args::parse();

    let stdin_channel = spawn_stdin_channel();
    let mut timer = Timer::new(args.count,
        args.focus.into(),
        args.pause.into(),
        args.long_pause.into()
    );

    loop {
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
        let text = format!("{} {} {}",
            timer.display_interval(),
            timer.display_duration(),
            timer.display_action()
        );
        let msg = format!("<span>{}</span>", text);
        let res = json!({"full_text": msg});
        let serialized = serde_json::to_string(&res).unwrap();
        println!("{}", serialized);
        sleep(Duration::from_secs(1));
        timer.update();
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
