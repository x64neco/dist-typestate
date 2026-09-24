# Distributed Typestate

Bring Rust's Typestate pattern to distributed systems using Versioned Capabilities and Compare-And-Swap (CAS).

## Why this crate?

Managing state across a distributed system (like microservices, job queues, or orchestration systems) is hard. Race conditions, split-brain scenarios, and stale writes are common bugs that lead to inconsistent data.

Usually, developers solve this by manually writing `if` statements and runtime checks before updating a database. If you forget a check, the system breaks.

This crate solves the problem by using **the Rust Compiler as your distributed system validator**. It brings the Typestate pattern out of a single process and applies it across your entire system.

### Key Features
* **Compile-Time Safety**: You literally cannot call `stop()` on a `Server` that isn't `Running`. The compiler will reject it.
* **Race-Condition Free**: State transitions consume ownership (`self`) and use optimistic concurrency control (CAS). Stale writes are safely caught as errors.
* **Declarative Macros**: Just define your state machine with `#[derive(DistributedTypestate)]`. All the boilerplate is generated for you.

## Quick Start

### 1. Define your State Machine

```rust
use dist_typestate::DistributedTypestate;

#[derive(DistributedTypestate)]
enum JobState {
    #[transition(claim => Claimed)]
    Queued,

    #[transition(start => Running)]
    Claimed,

    #[transition(complete => Completed, fail => Failed)]
    Running,

    Completed,

    #[transition(retry => Queued)]
    Failed,
}
```

### 2. Use it (Safely!)

```rust
use dist_typestate::sqlite::SqliteBackend;

// Initialize backend (SQLite for now, Redis coming soon)
let backend = SqliteBackend::in_memory()?;

// Create a new resource -> Returns Job<JobStateQueued>
let job = create_job_state(&backend, "job-001")?;

// Safe transition: Queued -> Claimed
let job = job.claim(&backend)?; 
// Safe transition: Claimed -> Running
let job = job.start(&backend)?; 

// Try to claim again? 
// job.claim(&backend); 
// ^^^ COMPILE ERROR: method `claim` not found for `Job<JobStateRunning>`!
```

### 3. Handling Runtime Races (Stale Capabilities)

If Node A and Node B both load the same state, and Node B transitions it first, Node A's capability becomes "stale".

```rust
// Load dynamic state from the backend into a Typed Handle
match load_job_state(&backend, "job-001")? {
    JobStateHandle::Running(job) => {
        // If someone else already stopped it, this returns an Err(StaleCapability)
        match job.complete(&backend) {
            Ok(completed_job) => println!("Success!"),
            Err(e) => println!("Race condition caught safely: {}", e),
        }
    }
    _ => println!("Not running!"),
}
```

## Backend Support

* **SQLite**: Fully supported (`SqliteBackend`)
* **Redis**: Planned (coming soon)
* **Custom**: You can write your own backend by implementing the `StateBackend` trait!

## License

Dual-licensed under MIT or Apache-2.0.
