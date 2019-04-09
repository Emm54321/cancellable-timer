# cancellable-timer

Crate that implements a timer with a `sleep` method that can be cancelled.

## Example

```rust
use std::time::Duration;
use cancellable_timer::*;

fn main() {
    let (mut timer, canceller) = Timer::new2().unwrap();

    // Spawn a thread that will cancel the timer after 2s.
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(2));
        println!("Stop the timer.");
        canceller.cancel();
    });

    println!("Wait 10s");
    let r = timer.sleep(Duration::from_secs(10));
    println!("Done: {:?}", r);
}
```

License: MIT/Apache-2.0
