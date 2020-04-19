use std::collections::HashMap;
use std::io::BufReader;
use std::time::Instant;

use super::redis::decoder::Decoder;

pub struct Stats {
    count: u64,
    started: Instant,
    queries: HashMap<String, u64>,
}

impl Stats {
    pub fn new() -> Stats {
        Stats {
            count: 0,
            started: Instant::now(),
            queries: HashMap::new(),
        }
    }

    pub fn count(&self) -> u64 {
        return self.count;
    }

    pub fn started_at(&self) -> Instant {
        return self.started;
    }

    pub fn accumulate_packet(&mut self, packet: etherparse::SlicedPacket) {
        let mut decoder = Decoder::new(BufReader::new(packet.payload));

        match decoder.decode_one() {
            Ok(value) => {
                let counter = self.queries.entry(value.abuse()).or_insert(0);
                *counter += 1;
            }
            Err(_) => {
                return;
            }
        };

        self.count += 1;
    }

    pub fn print_stats(&self) {
        let delta_secs = self.started.elapsed().as_secs_f64();

        println!(
            "Stats: {:?} queries, {:.2} QPS",
            self.count,
            (self.count as f64) / delta_secs
        );
        if self.queries.len() == 0 {
            return;
        }

        println!("");

        let mut sorted: Vec<(&String, &u64)> = Vec::new();

        for (key, count) in self.queries.iter() {
            if sorted.len() < 10 || count > sorted[9].1 {
                sorted.push((key, count));
            }
            sorted.sort_unstable_by(|a, b| b.1.partial_cmp(a.1).unwrap())
        }

        let mut ctr = 0;
        while ctr < sorted.len() && ctr < 10 {
            println!(
                "  {:8}  {:5.2}  {}",
                sorted[ctr].1,
                (*sorted[ctr].1 as f64) / delta_secs,
                sorted[ctr].0
            );

            ctr += 1;
        }

        println!("");
    }
}
