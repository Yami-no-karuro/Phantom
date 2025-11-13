use std::io;
use std::io::BufReader;
use std::io::BufRead;
use std::collections::HashMap;
use std::fs::File;

pub fn load_source(path: &str) -> Result<HashMap<String, bool>, io::Error> {
    let file: File = File::open(&path)?;
    let reader = BufReader::new(file);

    // The source file must contain a single value per row.
    // We'll use the HashMap lookup strategy to lower the complexity to O(1).
    let mut hashmap: HashMap<String, bool> = HashMap::new();
    for line in reader.lines() {
        hashmap.insert(line?, true);
    }

    return Ok(hashmap);
}
