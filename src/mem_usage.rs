use clap::Parser;
use lazy_static::lazy_static;
use regex::Regex;
use std::time::Duration;
use std::io::{self, Read};
use std::thread::sleep;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Refresh time in seconds
    #[arg(short, default_value_t = 1)]
    time: u64,

    /// Warning color for cpu load
    #[arg(short, default_value = "#FFA500")]
    warining_color: String,

    /// Critical color for cpu load
    #[arg(short, default_value = "#FF7373")]
    critical_color: String,

    /// Label written befor the memory usage value
    #[arg(short, default_value = "MEM ")]
    label: String
}

struct MemStats {
    used: u64,
    total: u64,
}

fn get_usage() -> MemStats {
    lazy_static! {
        static ref r: Regex = Regex::new(r"[a-zA-Z]*:[ ]*([0-9]*) kB").unwrap();
    }
    let file = match std::fs::File::open("/proc/meminfo") {
        Ok(file) => file,
        Err(err) => panic!("Could not open meminfo file: {err}"),
    };
    let mut file = std::io::BufReader::new(file);

    let mut file_content = "".to_string(); 
    match file.read_to_string(&mut file_content) {
        Ok(_) => {},
        Err(err) => { panic!("Could not read meminfo file: {err}")},
    };

    let file_content: Vec<u64> = file_content.
        split("\n").
        take(5).
        map(|s: &str| {
            r.captures(s).
                unwrap().
                get(1).
                unwrap().
                as_str().
                parse::<u64>().
                unwrap()
        }).
        collect();

    let mem_total = file_content[0];
    let mem_free = file_content[1];
    let buffers = file_content[3];
    let cached = file_content[4];

    return MemStats {
        used: mem_total - mem_free - buffers - cached,
        total: mem_total
    }
}

fn main() -> io::Result<()>{
    let args = Args::parse();

    let warn = args.warining_color;
    let crit = args.critical_color;

    loop {
        let mem_usage = get_usage();

        let used = mem_usage.used as f32 / 1024.0 / 1024.0;
        let total = mem_usage.total as f32 / 1024.0 / 1024.0;

        let p;
        if total > 0.0 {
            p = used / total * 100.0;
        } else {
            p = 0.0;
        }

        if p < 50.0 {
            println!("{}<span >{:2.1}G/{:2.1}G ({:2.0}%)</span>", args.label, used, total, p);
        } else if p < 80.0 {
            println!("{}<span color='{}'>{:2.1}G/{:2.1}G ({:2.0}%)</span>", args.label, warn, used, total, p);
        } else {
            println!("{}<span color='{}'>{:2.1}G/{:2.1}G ({:2.0}%)</span>", args.label, crit, used, total, p);
        }


        sleep(Duration::from_secs(args.time));
    }
}
