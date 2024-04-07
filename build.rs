use windres::Build;

fn main() {
    Build::new().compile("bms-kneeboard-server.rc").unwrap();
}
