use std::sync::Arc;
use std::thread;

use dist_typestate::DistributedTypestate;
use dist_typestate::backend::StateBackend;
use dist_typestate::sqlite::SqliteBackend;
use dist_typestate::error::TransitionError;

#[derive(DistributedTypestate)]
enum ServerState {
    #[transition(start => Starting)]
    Stopped,

    #[transition(started => Running)]
    Starting,

    #[transition(stop => Stopping, restart => Starting)]
    Running,

    #[transition(stopped => Stopped)]
    Stopping,
}

#[test]
fn two_threads_compete_for_transition() {
    let db_path = "test_multithread.db";
    let _ = std::fs::remove_file(db_path);

    let backend = Arc::new(SqliteBackend::new(db_path).unwrap());
    
    // macroによって生成されたcreate_server_stateを使用
    create_server_state(backend.as_ref(), "server-mt").unwrap();

    let b1 = Arc::clone(&backend);
    let b2 = Arc::clone(&backend);

    use std::sync::Barrier;
    let barrier = Arc::new(Barrier::new(2));
    let bar1 = Arc::clone(&barrier);
    let bar2 = Arc::clone(&barrier);

    let t1 = thread::spawn(move || -> Result<bool, TransitionError> {
        let handle = load_server_state(b1.as_ref(), "server-mt")?;
        bar1.wait();
        match handle {
            ServerStateHandle::Stopped(server) => {
                match server.start(b1.as_ref()) {
                    Ok(_) => Ok(true),
                    Err(TransitionError::StaleCapability { .. }) => Ok(false),
                    Err(e) => Err(e),
                }
            }
            _ => Ok(false),
        }
    });

    let t2 = thread::spawn(move || -> Result<bool, TransitionError> {
        let handle = load_server_state(b2.as_ref(), "server-mt")?;
        bar2.wait();
        match handle {
            ServerStateHandle::Stopped(server) => {
                match server.start(b2.as_ref()) {
                    Ok(_) => Ok(true),
                    Err(TransitionError::StaleCapability { .. }) => Ok(false),
                    Err(e) => Err(e),
                }
            }
            _ => Ok(false),
        }
    });

    let r1 = t1.join().unwrap().unwrap();
    let r2 = t2.join().unwrap().unwrap();

    assert_ne!(r1, r2, "exactly one thread should win the CAS race");
    println!("Thread 1 won: {r1}, Thread 2 won: {r2}");

    let (state, revision) = backend.load("server-mt").unwrap();
    assert_eq!(state, "starting");
    assert_eq!(revision, 2);

    let _ = std::fs::remove_file(db_path);
}
