use std::time::{Duration, Instant};

extern crate clap;
extern crate etherparse;
extern crate pcap;

use clap::{App, Arg};
use etherparse::SlicedPacket;
use pcap::{Capture, Device};

pub mod stats;
use stats::Stats;

pub mod redis;

fn main() {
    let config = App::new("redis-top")
        .version("0.1")
        .author("Mark Smith <mark@qq.is>")
        .about("Debug realtime traffic to a Redis server")
        .arg(
            Arg::with_name("interface")
                .short("i")
                .long("interface")
                .default_value("eth0")
                .value_name("DEVICE")
                .help("Which network device to capture on")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("snaplen")
                .short("s")
                .long("snaplen")
                .default_value("512")
                .value_name("BYTES")
                .help("How many bytes to read from outgoing packets")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("bpf")
                .long("bpf")
                .default_value("dst port 6379")
                .value_name("BPF_STRING")
                .help("BPF to select the right packets")
                .takes_value(true),
        )
        .get_matches();

    // Find the right interface and pass it to the reader
    let interface = config.value_of("interface").unwrap();
    for device in Device::list().unwrap() {
        if device.name == interface {
            read_from_device(config, device);
            return;
        }
    }
    println!("Error: interface {} not found", interface);
}

fn read_from_device(config: clap::ArgMatches, device: Device) {
    let mut cap = Capture::from_device(device)
        .unwrap()
        .promisc(false)
        .snaplen(config.value_of("snaplen").unwrap().parse().unwrap())
        .open()
        .unwrap();

    cap.filter(config.value_of("bpf").unwrap()).unwrap();

    let mut stats = Stats::new();
    let mut next_stats_at = stats
        .started_at()
        .checked_add(Duration::from_secs(10))
        .unwrap();

    while let Ok(packet) = cap.next() {
        match SlicedPacket::from_ethernet(packet.data) {
            Err(value) => println!("Error parsing packet: {:?}", value),
            Ok(value) => stats.accumulate_packet(value),
        }

        // If we're at time, output the accumulated stats
        if Instant::now() > next_stats_at {
            stats.print_stats();

            next_stats_at = Instant::now().checked_add(Duration::from_secs(10)).unwrap();
        }
    }
}
