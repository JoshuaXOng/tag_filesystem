use std::{collections::{HashMap, HashSet}, path::Path};

struct _A {
    _path: Box<Path>,
    working_labels: HashSet<String> 
}

struct Graph {
    relations: HashMap<String, String>,
    label_identifiers: HashSet<String>,
    file_identifiers: HashSet<String>,
}

impl Graph {
    fn new() -> Self {
        Self {
            relations: HashMap::new(),
            label_identifiers: HashSet::new(),
            file_identifiers: HashSet::new()
        }
    }

    fn from(root: &Path) {
         
    }
}

struct GraphLabel {
    identifier: String
}

struct GraphFile {
    identifier: String
}

#[test]
fn testi() {
    Graph::from(Path::new("/"));
}
