use cancellable_timer::*;
use std::io;
use std::time::Duration;

fn main() {
    let callback = |status: io::Result<()>| match status {
        Ok(_) => println!("Called"),
        Err(ref e) if e.kind() == io::ErrorKind::Interrupted => println!("Cancelled"),
        Err(e) => eprintln!("Error: {:?}", e),
    };

    println!("Run callback after 2s");
    Timer::after(Duration::from_secs(2), callback).unwrap();

    println!("Wait 4s");
    std::thread::sleep(Duration::from_secs(4));

    println!("Run callback after 4s");
    let canceller = Timer::after(Duration::from_secs(4), callback).unwrap();

    std::thread::sleep(Duration::from_secs(2));
    println!("Cancel timer.");
    canceller.cancel().unwrap();

    std::thread::sleep(Duration::from_secs(3));
}
