use std::fs::File;

fn main() {
    let config = Config::default();
    let file = File::open("wav.bt").unwrap();
    println!("{:?}", parse(&config, "wav.c").unwrap())
}
