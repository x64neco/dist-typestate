use dist_typestate::DistributedTypestate;
use dist_typestate::backend::StateBackend;
use dist_typestate::sqlite::SqliteBackend;

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = SqliteBackend::in_memory()?;

    let id = "server-001";
    // macroによって生成されたcreate関数
    let server = create_server_state(&backend, id)?;

    println!("正常系: Stopped → Starting → Running → Stopping → Stopped\n");

    println!("Created: {:?}", server);

    let server = server.start(&backend)?;
    println!("start(): {:?}", server);

    let server = server.started(&backend)?;
    println!("started(): {:?}", server);

    let server = server.stop(&backend)?;
    println!("stop(): {:?}", server);

    let server = server.stopped(&backend)?;
    println!("stopped(): {:?}", server);

    println!("\n競合テスト: StaleCapabilityの検出\n");

    backend.compare_and_transition(id, "stopped", 5, "running")?;
    println!("Backendで直接running/rev=6にした(別ノードの操作を模擬)");

    // Node Aがload
    let node_a = match load_server_state(&backend, id)? {
        ServerStateHandle::Running(s) => s,
        _ => panic!("should be running"),
    };
    println!("Node Aが取得: {:?}", node_a);

    // Node Bが先に遷移
    backend.compare_and_transition(id, "running", 6, "stopping")?;
    println!("Node Bが先にstopping/rev=7へ遷移");

    // Node Aがstale なcapabilityでstop()を試みる
    let result = node_a.stop(&backend);
    println!("Node Aがstop()を試行: {:?}", result);

    if matches!(result, Err(dist_typestate::error::TransitionError::StaleCapability { .. })) {
        println!("StaleCapability を正しく検出");
    } else {
        println!("予期しない結果: {:?}", result);
    }

    Ok(())
}
