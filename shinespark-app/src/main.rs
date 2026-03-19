extern crate shinespark;

fn main() {
    shinespark::trace::init().expect("trace init fail");
    println!("Hello, world!");
}
