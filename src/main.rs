extern crate regex;
extern crate hyper;
extern crate time;

use hyper::client::Client;
use std::io::prelude::*;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::env;
use std::collections::HashMap;
use regex::Regex;

fn main() {
    let (interface,mut macs) = read_arg();

    // TODO write pidfile

    // z-way port
    let port = 8083;

    // Build regexp for ARP scan lines
    /*
    00:10:95:de:ad:07 > ff:ff:ff:ff:ff:ff, ethertype ARP (0x0806), length 60: Probe 169.254.179.33, length 46
    00:10:95:de:ad:07 > ff:ff:ff:ff:ff:ff, ethertype ARP (0x0806), length 60: Probe 169.254.179.33, length 46
    5c:96:9d:77:90:43 (oui Unknown) > 24:a4:3c:b0:2b:27 (oui Unknown), ethertype ARP (0x0806), length 42: Reply 192.168.0.23 is-at 5c:96:9d:77:90:43 (oui Unknown), length 28
    24:a4:3c:b0:2b:27 (oui Unknown) > Broadcast, ethertype ARP (0x0806), length 42: Request who-has 192.168.0.1 (Broadcast) tell 192.168.0.21, length 28
    24:a4:3c:b0:2b:27 (oui Unknown) > Broadcast, ethertype ARP (0x0806), length 42: Request who-has 192.168.0.25 tell 192.168.0.21, length 28
    */
    let re_tcpdump = Regex::new(r"^(?x)
        ((?:[a-f0-9]{2}:){5}[a-f0-9]{2})
        \s
        >
        \s
        ((?:[a-f0-9]{2}:){5}[a-f0-9]{2})
        ,
        .+
        ").unwrap();

    // Start process and invoke tcpdump
    println!("Starting arp listen on {:?}",interface);
    let mut process = Command::new("tcpdump")
        .args(&["arp", "-e", "-t", "-l", "-n", "-i"])
        .arg(interface)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| {
            panic!("Failed to run tcpdump {}", e)
        });

    // Read from process
    if let Some(ref mut stdout) = process.stdout {

        let reader = BufReader::with_capacity(100,stdout);
        for line in reader.lines() {
            let line = line.unwrap();
            // Parse output from subprocess
            let capture = re_tcpdump.captures(&line);
            if capture.is_some() {
                let mac = capture.unwrap().at(1).unwrap();
                println!("Got data from tcpdump: {}", mac);
                let now  = time::now().to_timespec().sec;
                let limit = now - 30;
                // Got ARP request from MAC address
                if macs.contains_key(mac) {
                    if macs.get(mac).unwrap() < &limit {
                        macs.insert(mac.to_string(),now);
                        println!("Matched {}", mac);
                        make_reuqest(port,mac);
                    } else {
                        println!("Skipping {}", mac);
                    }
                }
            }
        }
    }

    // TODO delete pidfile
}

// Process arguments from commandline
fn read_arg() -> (String,HashMap<String,i64>) {
    let re_mac = Regex::new(r"^(?:[a-f0-9]{2}:){5}[a-f0-9]{2}$").unwrap();

    // Get interface
    let interface = env::args()
        .nth(1)
        .unwrap_or_else(||
            {
                println!("You need to supply the interface as first argument");
                std::process::exit(1);
            }
        );

    // Get MAC addresses
    let mut macs = HashMap::new();
    for index in 2..env::args().count() {
        let argn = env::args()
            .nth(index)
            .unwrap();
        if re_mac.is_match(&argn) {
            macs.insert(argn,0);
        } else {
            println!("Invalid MAC address supplied: {}",argn);
            std::process::exit(1);
        }
    }

    return (interface,macs);
}

// Make request ro controller
fn make_reuqest(port: i32,mac: &str) {
    let client = Client::new();

    let url = format!("http://localhost:{}/dash/{}",port,mac);
    println!("GET {}",url);
    let response = client.get(&url).send();
    match response {
        Ok(_r) => println!("OK accessing {}",url),
        Err(e) => println!("Error accessing {}: {}",url,e)
    }
}



