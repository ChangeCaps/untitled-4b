mod engine;

fn main() {
    let engine = engine::engine();

    engine.eval_file::<()>(engine::path(".shell.rhai")).unwrap();
}
