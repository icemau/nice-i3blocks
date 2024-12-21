use std::time::Duration;
use std::io::{self, BufRead};
use std::thread::sleep;

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

fn main() -> io::Result<()>{
    let cpu_stat = get_usage();

    let mut old_used = cpu_stat.get_used();
    let mut old_total = cpu_stat.get_total();

    loop {
        sleep(Duration::from_secs(1));
        let cpu_stat = get_usage();
        let used = cpu_stat.get_used();
        let total = cpu_stat.get_total();

        let p: f64 = 100.0 * (used - old_used) as f64 / (total - old_total) as f64;
        println!("{:6.2}", p);

        old_total = total;
        old_used = used;
    }
}
