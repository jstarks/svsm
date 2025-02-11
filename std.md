# Adding std support

This essentially amounts to porting Rust's `std` library to the new target.

This is easiest if coconut conforms to POSIX-ish behavior where practical. Don't
unnecessarily invent a new thing unless the benefit is real.

## `coconut-abi` crate

We need a crate (perhaps called `coconut-abi`) that provides the low-level
primitives for `std` to use. This crate will ultimately need to be in crates.io
for us to upstream the rust repo changes.

This is similar to the `syscall` crate, but lower level:

* No need for extra type safety constructs (e.g., `Obj` trait), since those will
  get in the way of `std`'s equivalents.
* Similarly, no owning RAII types (just expose `close()`, don't call it on
  `Drop` of some type).
* Doesn't have to be just syscall wrappers. E.g., `malloc` should be exposed
  here.

The code actually inside this crate should be minimal, though, since it will be
hard to change--it will be referenced by the Rust code with a specific version
number, and so it will require a new Rust compiler to update.

So, e.g., `malloc` should just be an `extern "C"` function that references some
library that you build as part of the Coconut SDK. Otherwise, we can't change
the `malloc` implementation or fix bugs without updating Rust.

Open question: what about straightforward syscall wrappers? Should they be
`extern "C"` functions, or can we just put the syscall invocation directly in
the crate? Do we need the flexibility to change the syscall ABI easily?

## Allocator

We need a general-purpose allocator. Easiest if this conforms to the POSIX API:

* `malloc`
* `posix_memalign`
* `calloc`
* `realloc`
* `free`

## File descriptors

How is the `Obj` handle thing different from file descriptors? Can we make them
more like POSIX file descriptors, so that we can use Rust's
`OwnedFd`/`AsFd`/etc. types and traits?

* Can we "reserve" 0, 1, and 2 for stdin/stdout/stderr, even if stdin does
  nothing and stdout/stderr are internally aliased? Today it looks like stdout
  is 0, which is an unnecessary divergence.
* Will `SYS_CLOSE` work for all objects?
* Can we support duplicating fds? Into a specific fd?

## Mutex

To support sync types `Mutex`, `RwLock`, `OnceLock`, etc., it will be easiest to
offer a futex-like API:

```rust
fn futex_wait(p: &AtomicU32, v: u32, timeout: Option<Duration>);
fn futex_wake(p: &AtomicU32);
fn futex_wake_all(p: &AtomicU32);
```

`futex_wait` should wait for `p.load() != v`, `futex_wake` should ensure that
one thread in `futex_wait` reevaluates the condition, `futex_wake_all` should
ensure that all threads reevaluate the condition.

For now, we can just implement this by spinning in user mode. But we should
provide an API for it (and maybe even a set of syscalls), so that we have the
flexibility to change this later without further changes to `std`.

## Time

We need a monotonic clock (i.e., `CLOCK_MONOTONIC`) to support waits with
timeouts, logging, etc. This can be a syscall initially, but it would be good to
implement this with a shared page + rdtsc or whatever.

If we can somehow support `CLOCK_REALTIME`, that would be interesting, but I am
not really sure what this means in a CVM world--you can never trust the value,
so what can you do with it? And seeding this time requires some
non-architectural communication with the host, unless you have networking and
NTP.

## Thread support

Core requirements:

* Threads share the virtual address space and file descriptor
  table.
* Thread-local storage (TLS) is supported, probably via the "initial-exec" model
  since we don't intend to support `dlopen`. I.e., have the thread start routine
  allocate the appropriately sized blob based on the ELF headers and store a
  pointer to it at `%fs:0`. Free this blob on thread exit.

If we want to support general-purpose threading, then we'll want to support
`std::thread::spawn`/`std::thread::Builder`. This means ideally we support
configurable stack sizes and thread names.

If we want to support threads not spawned by `std` but that _can_ exit before
the process exits, then we need to support Rust registering a function to run
when the thread exits (to run TLS destructors). I think we should probably
support this anyway, just in case. This could be via some special ELF construct
(a la Windows TLS) or via a function in `coconut-abi` that dynamically adds a
destructor to some table.

## Random numbers

Rust expects to use a cryptographic PRNG at startup to seed hash table hash
generation. We can work around this for now in various ways, but this will only
become more important over time (`std` will sooner or later get support for
generating cryptographically random numbers on demand).

## Misc

* Name the target (x86_64-unknown-coconut?)

## Unrelated ideas

* Can we change `SYS_EXEC` to take a file instead of a path?
* Can we use counted strings instead of null-terminated in the syscall
  interface? Seems silly to require an allocation to add a null terminator, and
  to add more complexity+geometric growth algorithm to the kernel side, given
  that we use counted strings on both sides.
* Let's add command line parameters to process startup.
* What about environment variables? Are these useful for anything?
* `SYS_CLOSE` should abort the process on invalid input.
* I'd suggest removing "ambient" state from the ABI--replace `SYS_OPEN` with
  `SYS_OPEN_AT` and require the process to be passed a root directory fd at
  start.
