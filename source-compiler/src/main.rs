use asttoir;
use ir;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

fn main() {
    let contents = asttoir::read_from_file(None);
    let a = ir::Func::new_with_params_and_result(&[ir::VarType::Any], ir::VarType::Any);
}