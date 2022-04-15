#[test]
fn test_goglob_proc_macro() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-goglob-gotest-pass.rs");
    t.compile_fail("tests/02-goglob-gotest-fail.rs");
}
