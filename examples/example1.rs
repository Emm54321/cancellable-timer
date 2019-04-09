use cancellable_timer::*;
use std::io;
use std::time::Duration;

fn main() {
    let (mut timer, canceller) = Timer::new2().unwrap();

    println!("Wait 2s, uninterrupted.");
    let r = timer.sleep(Duration::from_secs(2));
    println!("Done: {:?}", r);

    println!("Wait 2s, cancelled at once.");
    canceller.cancel().unwrap();
    let r = timer.sleep(Duration::from_secs(2));
    println!("Done {:?}", r);

    println!("Wait 2s, not cancelled.");
    let r = timer.sleep(Duration::from_secs(2));
    println!("Done {:?}", r);

    println!("Wait 10s, cancel after 2s");
    let canceller2 = canceller.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(2));
        canceller2.cancel().unwrap();
    });
    match timer.sleep(Duration::from_secs(10)) {
        Err(ref e) if e.kind() == io::ErrorKind::Interrupted => println!("Cancelled"),
        x => panic!("{:?}", x),
    };
}
