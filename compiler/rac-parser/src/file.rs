use std::fs::File;
use rac_diagnostics::Report;

pub fn init_src<'a> (fname: &'static str) -> &'a [u8] {
    let mut f = File::open(fname);
    todo!()
}
