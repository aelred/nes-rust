use nes_rust::runtime;

fn main() {
    if let Err(e) = runtime::run(log::Level::Info) {
        log::error!("{}", e);
    }
}
