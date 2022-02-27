extern crate getopts;
extern crate portmidi as pm;

use getopts::Options;

use std::env;
use std::thread::sleep;
use std::time::Duration;

const BUFFER_SIZE:usize = 1024;

fn get_input_port(context: &pm::PortMidi, device_id: i32) -> Result<pm::InputPort, pm::Error> {
    let info = context.device(device_id)?;
    let inport = context.input_port(info, BUFFER_SIZE)?;
    Ok(inport)
}

fn get_output_port(context: &pm::PortMidi, device_id: i32) -> Result<pm::OutputPort, pm::Error> {
    let info = context.device(device_id)?;
    let outport = context.output_port(info, BUFFER_SIZE)?;
    Ok(outport)
}

fn print_devices(portmidi: &pm::PortMidi) {
    let devices = match portmidi.devices() {
        Ok(d) => d,
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

fn read_write(inport: &pm::InputPort, monitor: bool, outport: &mut Option<pm::OutputPort>) {
    let timeout_ms = 10;

    let timeout = Duration::from_millis(timeout_ms);
    while let Ok(_) = inport.poll() {
        if let Ok(Some(events)) = inport.read_n(BUFFER_SIZE) {
            for event in events {
                if monitor {
                    println!("{:?}", event);
                }
                if let Some(ref mut o) = outport {
                    o.write_message(event.message).unwrap();
                }
            }
        }
        sleep(timeout);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("p", "print", "print the available MIDI devices");
    opts.optflag(
        "m",
        "monitor",
        "print incoming MIDI messages from the specified input",
    );
    opts.optopt("i", "input", "input device number", "INPUT");
    opts.optopt("o", "output", "output device number", "OUTPUT");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let context = match pm::PortMidi::new() {
        Ok(c) => c,
        Err(e) => {
            panic!("{}", e.to_string())
        }
    };

    if matches.opt_present("p") {
        print_devices(&context);
    }

    if let Some(ref i) = matches.opt_str("i") {
        let monitor = matches.opt_present("m");
        let input_device = i.parse().unwrap();

        let inport = match get_input_port(&context, input_device) {
            Ok(p) => p,
            Err(e) => {
                panic!("{}", e.to_string())
            }
        };

        let mut outport = None;
        if let Some(ref o) = matches.opt_str("o") {
            let k = o.parse().unwrap();
            outport = Some(get_output_port(&context, k).unwrap());
        }

        read_write(&inport, monitor, &mut outport);
    }
}
