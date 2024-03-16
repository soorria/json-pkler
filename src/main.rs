fn main() {
    println!(
        "Hello, world! {:?}",
        "-1.2e+3".chars().collect::<Vec<_>>()[0..10]
            .iter()
            .collect::<String>()
    );
}
