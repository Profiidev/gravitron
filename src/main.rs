use gravitron::engine::Engine;

fn main() {
  let engine = Engine::builder_client().with_state(()).build().unwrap();
  engine.run();
}
