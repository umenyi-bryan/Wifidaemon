use anyhow::Result;
use pcap::{Capture, Device};
use serde::Serialize;
use std::env;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const BANNER: &str = r#"
           __        ___ _     _
 _      _  \ \      / (_) | __| | ___  ___| |_ _ __ ___  _ __
| | | | |  \ \ /\ / /| | |/ _` |/ _ \/ __| __| '__/ _ \| '_ \
| |_| | |   \ V  V / | | | (_| |  __/\__ \ |_| | | (_) | | | |
 \__,_|_|    \_/\_/  |_|_|\__,_|\___||___/\__|_|  \___/|_| |_|
                      WiFiDAEMON 😈 v0.1.0
              "Silent Guardian of the Airwaves""#;

#[derive(Serialize)]
struct BeaconSummary {
    ts: u128,
    note: String,
    len: usize,
}

fn ts_now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

fn print_banner() {
    println!("{}", BANNER);
}

fn main() -> Result<()> {
    env_logger::init();
    print_banner();

    let args: Vec<String> = env::args().collect();
    let mut pcap_file: Option<PathBuf> = None;
    let mut out_json: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--pcap-file" => {
                if i + 1 < args.len() {
                    pcap_file = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--output-json" => {
                if i + 1 < args.len() {
                    out_json = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    if let Some(pcap_path) = pcap_file {
        println!("Running in PCAP replay mode: {:?}", pcap_path);
        let mut cap = Capture::from_file(pcap_path)?;
        let mut writer: Option<std::fs::File> =
            out_json.map(|p| std::fs::File::create(p).unwrap());
        while let Ok(packet) = cap.next_packet() {
            let rec = BeaconSummary {
                ts: ts_now_ms(),
                note: "beacon/placeholder".to_string(),
                len: packet.header.len as usize,
            };
            let j = serde_json::to_string(&rec)?;
            println!("{}", j);
            if let Some(w) = writer.as_mut() {
                use std::io::Write;
                writeln!(w, "{}", j)?;
            }
        }
    } else {
        println!("No --pcap-file provided. Attempting live capture...");

        match Device::lookup() {
            Ok(Some(device)) => {
                println!("Using device: {}", device.name);
                let dev_name = device.name.clone();
                let mut cap = Capture::from_device(dev_name.as_str())
                    .expect("Failed to open device")
                    .promisc(true)
                    .immediate_mode(true)
                    .open()
                    .expect("Could not start capture");

                cap.filter("type mgt subtype beacon", true)
                    .expect("Failed to apply filter");

                while let Ok(packet) = cap.next_packet() {
                    let rec = BeaconSummary {
                        ts: ts_now_ms(),
                        note: "beacon".to_string(),
                        len: packet.header.len as usize,
                    };
                    println!("{}", serde_json::to_string(&rec).unwrap());
                }
            }

            Ok(None) => {
                println!("⚠️  No network interface found (None returned). Running in demo mode...");
                for n in 0..10 {
                    let rec = BeaconSummary {
                        ts: ts_now_ms(),
                        note: format!("demo-beacon-{}", n),
                        len: 128,
                    };
                    println!("{}", serde_json::to_string(&rec).unwrap());
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            }

            Err(_) => {
                println!("⚠️  Device lookup failed. Running in demo mode...");
                for n in 0..10 {
                    let rec = BeaconSummary {
                        ts: ts_now_ms(),
                        note: format!("demo-beacon-{}", n),
                        len: 128,
                    };
                    println!("{}", serde_json::to_string(&rec).unwrap());
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            }
        }
    }

    Ok(())
}
