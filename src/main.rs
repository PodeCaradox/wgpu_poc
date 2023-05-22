//! A shader that renders a mesh multiple times in one draw call
//use crate::draw::instancing_pipline::run;
use castle_sim::run;

fn main() {
    pollster::block_on(run(1600,1000));
}
