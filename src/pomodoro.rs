use std::time::{Duration, SystemTime};
use std::thread::sleep;
use std::{io, usize};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::TryRecvError;
use std::thread;
use serde::{Serialize, Deserialize};

/*
{
    "":"",
    "separator":true,
    "separator_block_width":15,
    "name":"test",
    "command":"/home/icemau/projects/i3blocks-nice-blocks/target/debug/pomodoro",
    "markup":"pango",
    "interval":"persist",
    "format":"json",
    "full_text":"<span>Empty</span>",
    "button":1,
    "modifiers":[],
    "x":1763,
    "y":1431,
    "relative_x":13,
    "relative_y":12,
    "output_x":1763,
    "output_y":1431,
    "width":40,
    "height":21}"
}
*/
#[derive(Serialize, Debug)]
struct S {
    full_text: String
}

#[derive(Serialize, Deserialize, Debug)]
struct R {
    button: i32,
}

struct Timer {
    cur_interval: i32,
    start_time: Option<SystemTime>,
    intervals: Vec<Duration>,
}

impl Timer {
    fn new() -> Self {
        return Self {
            cur_interval: 0,
            start_time: None,
            intervals: vec![
                Duration::from_secs(10),
                Duration::from_secs(15),
                Duration::from_secs(25 * 60),
                Duration::from_secs(5 * 60),
                Duration::from_secs(25 * 60),
                Duration::from_secs(15 * 60),
            ]
        }
    }

    fn start(&mut self) {
        self.start_time = Some(SystemTime::now());
    }

    fn is_done(&self) -> bool {
        let now = SystemTime::now();
        if let Some(start_time) = self.start_time {
            return now.duration_since(start_time).unwrap() >= self.intervals[self.cur_interval as usize];
        } else {
            return false;
        }
    }

    fn next(&mut self) {
        self.start_time = None;
        self.cur_interval += 1;
    }

    fn display(&self) -> String {
        let now = SystemTime::now();
        if let Some(start_time) = self.start_time {
            let since = now.duration_since(start_time).unwrap();
            if self.intervals[self.cur_interval as usize] < since {
                return format!("{:02}:{:02}", 0, 0);
            } else {
                let diff = self.intervals[self.cur_interval as usize] - since; 
                let sec = diff.as_secs_f64() as i32;
                return format!("{:02}:{:02}", sec / 60, sec % 60)
            }
        } else {
            let diff = self.intervals[self.cur_interval as usize]; 
            let sec = diff.as_secs_f64() as i32;
            return format!("{:02}:{:02}", sec / 60, sec % 60)
        }
    }
}

fn main() -> io::Result<()>{
    let stdin_channel = spawn_stdin_channel();
    let mut timer = Timer::new();
    loop {
        match stdin_channel.try_recv() {
            Ok(key) => {
                let r: R = serde_json::from_str(&key).unwrap();
                if r.button == 1 {
                    timer.start()
                }
            }
            Err(TryRecvError::Empty) => {},
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        };
        let msg = S{full_text: format!("<span>{}</span>", timer.display())};
        let serialized = serde_json::to_string(&msg).unwrap();
        println!("{}", serialized);
        if timer.is_done(){
            timer.next()
        };
        sleep(Duration::from_millis(500));
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
