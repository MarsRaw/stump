# Stump
Stump is a *very* simple console logging library.

## Environment variable control:
```bash
STUMP_LOG_AT_LEVEL = DEBUG | INFO | WARN | ERROR

STUMP_LOG_DATETIME_FORMAT = "%Y-%m-%d %H:%M:%S%.3f"
```

See https://docs.rs/chrono/latest/chrono/ for date & time formatting.

## Logging
```rust
debug!("Kevin is");
info!("bad at");
warn!("writing documentation");
error!("for OSS projects");
```

## Task Completion Messages
Stump also provides functions for printing task completion status messages.

```rust
stump::print_done("Some process finished");
// Some process finished                                    [ DONE ]

stump::print_warn("Some process with warnings");
// Some process with warnings                               [ WARN ]

stump::print_fail("Some process failed");
// Some process failed                                      [ FAIL ]
```

## Overriding Stdout
When integrating stump with another CLI library, such as `indicatif`, you can provide another means of printing, such as to
route the output through their print method:

```rust
use indicatif::ProgressBar;
use stump;

lazy_static! {
    static ref PB: ProgressBar = ProgressBar::new(1);
}

fn main() {

    stump::set_print(|s| {
        PB.println(s);
    });

}


```