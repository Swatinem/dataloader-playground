fn main() {
    cynic_codegen::register_schema("library")
        .from_sdl_file("schemas/library.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
