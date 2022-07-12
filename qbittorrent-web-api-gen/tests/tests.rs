#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    // --- Auth ---
    t.pass("tests/login.rs");
    t.pass("tests/logout.rs");

    // --- Parameters ---
    t.pass("tests/without_parameters.rs");
    // t.pass("tests/with_parameters.rs");
    t.pass("tests/default_parameters.rs");

    // --- Return types ---
    t.pass("tests/return_type.rs");
    t.pass("tests/return_type_with_optional_params.rs");
    t.pass("tests/return_type_enum.rs");

    // --- Misc ---
    t.pass("tests/add_torrent.rs");
    t.pass("tests/another_struct_name.rs");
    t.pass("tests/access_impl_types.rs");
}
