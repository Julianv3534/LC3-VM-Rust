use std::env;
use std::process;

use lc3_vm::{InputBufferingGuard, Vm, VmError};

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        process::exit(1);
    }
}

fn run() -> Result<(), VmError> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: {} [image-file1] ...", args[0]);
        process::exit(2);
    }

    let mut vm = Vm::new();
    for image in &args[1..] {
        vm.load_image_file(image)?;
    }

    let _input_guard = InputBufferingGuard::disable()?;
    vm.run()?;

    Ok(())
}
