extern crate getopts;
extern crate portmidi as pm;

use getopts::Options;

use std::error::Error;
use std::time::Duration;
use std::thread::{spawn, sleep};
use std::sync::mpsc;
use std::env;


fn get_input_port(context: &pm::PortMidi, device_id: i32) -> Result<pm::InputPort, pm::Error> {
    let info = context.device(device_id)?;
    let inport = context.input_port(info, 1024)?;
    Ok(inport)
}

fn get_output_port(context: &pm::PortMidi, device_id: i32) -> Result<pm::OutputPort, pm::Error> {
    let info = context.device(device_id)?;
    let outport = context.output_port(info, 1024)?;
    Ok(outport)
}

fn stream(inport: &pm::InputPort, tx: mpsc::Sender<pm::MidiMessage>) -> Result<(), Box<Error>> {
    let timeout = Duration::from_millis(10);
    while let Ok(_) = inport.poll() {
        if let Ok(Some(events)) = inport.read_n(1024) {
            for event in events {
                let tx = tx.clone();
                spawn(move || {
                    tx.send(event.message);
                });
            }
        }
        sleep(timeout);
    }
    Ok(())
}

fn print_devices(portmidi: &pm::PortMidi) {
    let devices = match portmidi.devices() {
        Ok(d) => {d},
        Err(_) => {
            println!("Encountered error trying to get devices");
            return; // not panicking here since it's for info purposes
        }
    };
    for dev in devices { 
        println!("{}", dev);
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("p", "print", "print the available MIDI devices");
    opts.optflag("m", "monitor", "print incoming MIDI messages from the specified input");
    opts.optopt("i", "input", "input device", "INPUT");
    opts.optopt("o", "output", "output device", "OUTPUT");
    

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => {m}
        Err(f) => {panic!(f.to_string())}
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let context = match pm::PortMidi::new() {
        Ok(c) => {c}
        Err(e) => {panic!(e.to_string())}
    };

    if matches.opt_present("p") {
        print_devices(&context);
        return;
    }

    let monitor = matches.opt_present("m");

    let in_device = matches.opt_str("i");
    if let Some(ref s) = in_device {
        let (tx, rx) = mpsc::channel();
        let d = s.parse().unwrap();
        let inport = get_input_port(&context, d).unwrap();
        spawn(move || {
            match stream(&inport, tx) {
                Ok(_) => {},
                Err(e) => {panic!(e.to_string())},
            };
        });

        let mut outport = None;
        let out_device = matches.opt_str("o");
        if let Some(ref o) = out_device {
            let k = o.parse().unwrap();
            outport = Some(get_output_port(&context, k).unwrap());
        }

        loop {
            let message = rx.recv().unwrap();
            if monitor {
                println!("{}",message);
            }
            if let Some(ref mut o) = outport {
                o.write_message(message).unwrap();
            }
        }
    }
}
