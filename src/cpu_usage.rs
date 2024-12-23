use clap::Parser;
use std::time::Duration;
use std::io::{self, BufRead};
use std::thread::sleep;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Refresh time in seconds
    #[arg(short, default_value_t = 1)]
    time: u64,

    /// Floating point precision of cpu usage
    #[arg(short, default_value_t = 2)]
    decimal: usize,
    
    /// Warning color for cpu load
    #[arg(short, default_value = "#FFA500")]
    warining_color: String,

    /// Critical color for cpu load
    #[arg(short, default_value = "#FF7373")]
    critical_color: String,

    /// Label written befor the cpu usage value
    #[arg(short, default_value = "CPU ")]
    label: String
}

struct CpuStat {
    user: u64,
    nice: u64,
    sys: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    sirq: u64,
    steal: u64,
    guest: u64,
    nguest: u64,
}

impl CpuStat {
    fn get_total(&self) -> u64 {
        return self.get_used() + self.idle + self.iowait; 
    }

    fn get_used(&self) -> u64 {
        return self.user + self.nice + self.sys + self.irq + self.sirq + self.steal + self.guest + self.nguest
    }
}

fn get_usage() -> CpuStat {
    let file = match std::fs::File::open("/proc/stat") {
        Ok(file) => file,
        Err(err) => panic!("Could not open stat file: {err}"),
    };
    let mut file = std::io::BufReader::new(file);

    let mut buffer = Vec::new();

    match file.read_until(b'\n', &mut buffer) {
        Ok(_) => { },
        Err(err) => panic!("Could not read stat file: {err}"),
    };

    let line = match String::from_utf8(buffer) {
        Ok(line) => line,
        Err(err) => panic!("Could not convert buffer to string: {err}"),
    };

    let line = line.trim();

    let s: Vec<u64> = line.
        split(" ").
        skip(2).
        map(|s: &str| {
            match s.parse() {
                Ok(u) => u,
                Err(err) => panic!("Could not parse string to unsiged integer: {err}"),
            }
        }).
        collect();

    if s.len() !=  10 {
        panic!("Unexpected amount of values. Expected 10 got {}", s.len())
    }

    return CpuStat{
        user: s[0],
        nice: s[1],
        sys: s[2],
        idle: s[3],
        iowait: s[4],
        irq: s[5],
        sirq: s[6],
        steal: s[7],
        guest: s[8],
        nguest: s[9],
    }
}

static RED: &str = "#FF7373";
static ORANGE: &str = "#FFA500";

fn main() -> io::Result<()>{
    let args = Args::parse();

    let cpu_stat = get_usage();

    let mut old_used = cpu_stat.get_used();
    let mut old_total = cpu_stat.get_total();

    loop {
        let cpu_stat = get_usage();
        let used = cpu_stat.get_used();
        let total = cpu_stat.get_total();

        let p: f64;
        if total - old_total == 0 {
            p = 0.0
        } else {
            p = 100.0 * (used - old_used) as f64 / (total - old_total) as f64;
        }

        if p < 50.0 {
            println!("{}<span>{:6.2$}</span>",args.label, p, args.decimal);
        } else if p < 80.0 {
            println!("{}<span color='{}'>{:6.3$}</span>", args.label, ORANGE, p, args.decimal);
        } else {
            println!("{}<span color='{}'>{:6.3$}</span>", args.label, RED, p, args.decimal);
        }

        old_total = total;
        old_used = used;

        sleep(Duration::from_secs(args.time));
    }
}
