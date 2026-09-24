/// derive macroを使った宣言的な状態機械定義のテスト
use dist_typestate::DistributedTypestate;
use dist_typestate::backend::StateBackend;
use dist_typestate::error::TransitionError;
use dist_typestate::sqlite::SqliteBackend;

// JobState enumからJob<State>ラッパー型が生成される
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

// メソッドを追加可能
impl Job<JobStateRunning> {
    pub fn process(&self) {
        println!("Processing job: {}", self.id());
    }
    pub fn progress(&self, backend: &impl StateBackend) -> Result<f64, TransitionError> {
        // self.id(),self.revision()はmacroが生成済み
        let (_state, _rev) = backend.load(self.id())?;
        Ok(0.42)
    }
}


#[test]
fn macro_generated_happy_path() {
    let backend = SqliteBackend::in_memory().unwrap();

    let job = create_job_state(&backend, "job-001").unwrap();
    assert_eq!(job.revision(), 1);

    let job = job.claim(&backend).unwrap();
    assert_eq!(job.revision(), 2);

    let job = job.start(&backend).unwrap();
    job.progress(&backend).unwrap();
    assert_eq!(job.revision(), 3);

    let job = job.complete(&backend).unwrap();
    assert_eq!(job.revision(), 4);

    let (state, rev) = backend.load("job-001").unwrap();
    assert_eq!(state, "completed");
    assert_eq!(rev, 4);
}

#[test]
fn macro_generated_fail_and_retry() {
    let backend = SqliteBackend::in_memory().unwrap();
    let job = create_job_state(&backend, "job-002").unwrap();

    let job = job.claim(&backend).unwrap();
    let job = job.start(&backend).unwrap();

    let job = job.fail(&backend).unwrap();
    assert_eq!(job.revision(), 4);

    let job = job.retry(&backend).unwrap();
    assert_eq!(job.revision(), 5);

    let (state, _) = backend.load("job-002").unwrap();
    assert_eq!(state, "queued");
}

#[test]
fn macro_generated_load_handle() {
    let backend = SqliteBackend::in_memory().unwrap();
    backend.create("job-003", "running").unwrap();

    let handle = load_job_state(&backend, "job-003").unwrap();

    match handle {
        JobStateHandle::Running(job) => {
            let _completed = job.complete(&backend).unwrap();
        }
        _ => panic!("expected Running"),
    }
}

#[test]
fn macro_generated_stale_capability() {
    let backend = SqliteBackend::in_memory().unwrap();
    let job = create_job_state(&backend, "job-004").unwrap();

    backend
        .compare_and_transition("job-004", "queued", 1, "claimed")
        .unwrap();

    let result = job.claim(&backend);
    assert!(matches!(
        result,
        Err(TransitionError::StaleCapability { .. })
    ));
}


// 以下コンパイルエラーになる
//
// fn compile_error_test() {
//     let backend = SqliteBackend::in_memory().unwrap();
//     let job = create_job_state(&backend, "x").unwrap();
//     // jobはJob<JobStateQueued>なのでstart()は存在しない
//     job.start(&backend);  // ERROR: method not found
// }
