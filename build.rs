extern crate embed_resource;

fn main() {
    if cfg!(target_os = "windows") {
        embed_resource::compile("embed_icon.rc", embed_resource::NONE);
    }
}
