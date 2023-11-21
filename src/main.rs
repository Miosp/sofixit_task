use crate::data_gen::RandomGen;

mod data_gen;

fn main() {
    let mut rng = rand::thread_rng();
    let data = data_gen::FakeData::random(&mut rng);
    println!("{}", serde_json::to_string(&data).unwrap());
}
