use deno_core::error::AnyError;
use deno_core::futures::FutureExt;
use deno_core::op;
use deno_core::{FsModuleLoader, ModuleLoader};
use deno_runtime::deno_broadcast_channel::InMemoryBroadcastChannel;
use deno_runtime::deno_web::BlobStore;
use deno_runtime::permissions::Permissions;
use deno_runtime::worker::MainWorker;
use deno_runtime::worker::WorkerOptions;
use deno_runtime::BootstrapOptions;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

pub async fn render() -> anyhow::Result<()> {
    let module_loader = Rc::new(Loader::new());
    let create_web_worker_cb = Arc::new(|_| {
        todo!("Web workers are not supported in the example");
    });
    let web_worker_event_cb = Arc::new(|_| {
        todo!("Web workers are not supported in the example");
    });

    let options = WorkerOptions {
        bootstrap: BootstrapOptions {
            args: vec![],
            cpu_count: 1,
            debug_flag: false,
            enable_testing_features: false,
            location: None,
            no_color: false,
            is_tty: false,
            runtime_version: "x".to_string(),
            ts_version: "x".to_string(),
            unstable: false,
            user_agent: "hello_runtime".to_string(),
            inspect: false,
        },
        extensions: vec![],
        unsafely_ignore_certificate_errors: None,
        root_cert_store: None,
        seed: None,
        source_map_getter: None,
        format_js_error_fn: None,
        web_worker_preload_module_cb: web_worker_event_cb.clone(),
        web_worker_pre_execute_module_cb: web_worker_event_cb,
        create_web_worker_cb,
        maybe_inspector_server: None,
        should_break_on_first_statement: false,
        module_loader,
        npm_resolver: None,
        get_error_class_fn: Some(&get_error_class_name),
        cache_storage_dir: None,
        origin_storage_dir: None,
        blob_store: BlobStore::default(),
        broadcast_channel: InMemoryBroadcastChannel::default(),
        shared_array_buffer_store: None,
        compiled_wasm_module_store: None,
        stdio: Default::default(),
    };

    let js_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("test.js");
    let main_module = deno_core::resolve_path(&js_path.to_string_lossy())?;
    let permissions = Permissions::allow_all();

    let mut worker = MainWorker::bootstrap_from_options(main_module.clone(), permissions, options);

    let val = worker
        .js_runtime
        .execute_script("test", "new Promise((res) => res())")?;

    worker.js_runtime.resolve_value(val).await?;

    worker
        .js_runtime
        .op_state()
        .borrow_mut()
        .put(Modules(Vec::default()));
    // let id = worker
    //     .js_runtime
    //     .load_main_module(&main_module, Some(include_str!("../test.js").to_string()))
    //     .await?;
    // for _ in 0..2 {
    //     worker
    //         .js_runtime
    //         .op_state()
    //         .borrow_mut()
    //         .put(Modules(Vec::default()));

    //     worker.evaluate_module(id).await?;
    //     // worker.execute_main_module(&main_module).await?;
    //     worker.run_event_loop(false).await?;

    //     let modules = worker.js_runtime.op_state().borrow_mut().take::<Modules>();

    //     println!("Hello {:?}", modules);
    // }

    std::thread::sleep(std::time::Duration::from_millis(5000));

    Ok(())
}
