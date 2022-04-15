use goglob::{glob, GlobPattern};

const PATTERN_01: GlobPattern = glob!("[]a]");
const PATTERN_02: GlobPattern = glob!("[-]");
const PATTERN_03: GlobPattern = glob!("[x-]");
const PATTERN_04: GlobPattern = glob!("[-x]");
const PATTERN_05: GlobPattern = glob!("\\");
const PATTERN_06: GlobPattern = glob!("[a-b-c]");
const PATTERN_07: GlobPattern = glob!("[");
const PATTERN_08: GlobPattern = glob!("[^");
const PATTERN_09: GlobPattern = glob!("[^bc");
const PATTERN_10: GlobPattern = glob!("a[");
const PATTERN_11: GlobPattern = glob!("a/b[");

fn main() {}