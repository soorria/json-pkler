use json_pkler::parse_json;

fn main() {
    const BENCH_DATA_001: &str = include_str!("../bench_data/006_beeg_formatted.json");
    let _ = parse_json(BENCH_DATA_001).unwrap();
}
