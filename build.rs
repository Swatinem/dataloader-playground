fn main() {
    cynic_codegen::register_schema("loader")
        .from_sdl_file("schemas/loader.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
