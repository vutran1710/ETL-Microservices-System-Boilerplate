use database::tier_1::ExampleTable;

pub fn main() {
    let table = ExampleTable {
        id: "example_id".to_string(),
    };
    println!("{:?}", table);
}
