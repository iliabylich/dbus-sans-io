## DBus Sans I/O

At its core this library provides IO-free interfaces for communicating with DBus (`fsm` module).

On top of this the library provides 3 reference implementations:

1. `BlockingConnection` (requires `blocking` feature enabled, uses blocking `read` and `write`)
2. `PollConnection` (requires `poll` feature enabled, uses `poll` + `read` + `write`)
3. `IoUringConnection` (requires `io-uring` feature enabled, uses `io_uring` for acquiring a socket, connecting to dbus and doing both reads and writes)
