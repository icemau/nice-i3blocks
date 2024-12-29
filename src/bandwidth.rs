use clap::Parser;
use std::time::{SystemTime, Duration};
use std::collections::HashMap;
use std::io::{self, Read};
use std::thread::sleep;
use lazy_static::lazy_static;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Refresh time in seconds
    #[arg(short, default_value_t = 1)]
    time: u64,

    /// Label written befor the memory usage value
    #[arg(short, default_value = "NET ")]
    label: String
}

struct NetStats {
    rx: u64,
    tx: u64,
}

fn get_values() -> NetStats {
    let file = match std::fs::File::open("/proc/net/dev") {
        Ok(file) => file,
        Err(err) => panic!("Could not open network file: {err}"),
    };
    let mut file = std::io::BufReader::new(file);

    let mut file_content = "".to_string(); 
    match file.read_to_string(&mut file_content) {
        Ok(_) => {},
        Err(err) => { panic!("Could not read network file: {err}")},
    };

    let mut net_stats = NetStats{rx: 0, tx: 0};

    for line in file_content.split("\n").skip(2).into_iter() {
        if line == "" {
            continue;
        }

        let line_split: Vec<&str> = line.splitn(2, ":").collect();
        if line_split.len() != 2 {
            panic!("Unexpected line: {}", line)
        }
        let int_name = line_split[0];

        if int_name == "lo" {
            continue;
        }

        let int_stats = line_split[1].
            split(" ").
            filter(|v| *v != "").
            map(|v| v.parse::<u64>().unwrap()).
            collect::<Vec<u64>>();

        if int_stats.len() != 16 {
            panic!("Unexpected amount of interface values")
        }

        net_stats.rx += int_stats[0];
        net_stats.tx += int_stats[8];
    }
    net_stats
}

lazy_static!{
    static ref UNITS: HashMap<i32, &'static str> = [
        (0, " B"),
        (1, "KB"),
        (2, "MB"),
        (3, "GB"),
        (4, "TB"),
    ].iter().copied().collect();
}

fn main() -> io::Result<()>{
    let args = Args::parse();

    let divisor = 1024.0;

    let mut before = SystemTime::now();
    let mut old_net_stat = get_values();

    loop {
        let net_stat = get_values();

        let now = SystemTime::now();
        let elapsed= now.duration_since(before).unwrap().as_secs_f32();

        let mut tx = (net_stat.tx - old_net_stat.tx) as f32 / elapsed;
        let mut txd = 0;
        while tx >= divisor {
            tx = tx / divisor;
            txd += 1;
        }

        let mut rx = (net_stat.rx - old_net_stat.rx) as f32 / elapsed as f32;
        let mut rxd = 0;
        while rx >= divisor {
            rx = rx / divisor;
            rxd += 1;
        }

        println!("{} <span>{:6.1}{}/s {:6.1}{}/s</span>", args.label, rx, UNITS[&rxd], tx, UNITS[&txd]);
        old_net_stat = net_stat;
        before = now;
        sleep(Duration::from_secs(args.time));
    }
}
