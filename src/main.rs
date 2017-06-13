extern crate shadow_cost_rust;
use shadow_cost_rust::*;
use std::env;

fn main() {
    let mut argv = env::args();
    argv.next();
    let deck_size: u32 = argv.next().unwrap_or("30".to_string()).parse().unwrap();
    let turn_max: u32 = argv.next().unwrap_or("10".to_string()).parse().unwrap();
    let loop_count: u32 = argv.next().unwrap_or("100".to_string()).parse().unwrap();
    let trial_count: u32 = argv.next().unwrap_or("100".to_string()).parse().unwrap();

    shadow_cost::CostSim::new(deck_size, turn_max).search_deck(loop_count, trial_count);
}
