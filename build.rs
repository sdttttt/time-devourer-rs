fn main() {
    println!("cargo:rerun-if-changed=timer.rc");
    println!("cargo:rerun-if-changed=resource.h");
    println!("cargo:rerun-if-changed=icon.ico");
    embed_resource::compile("timer.rc", embed_resource::NONE);
}
