use std::collections::HashMap;
use std::path::PathBuf;
use std::io;

/// The dependencies of each template
#[derive(Default, Debug)]
pub struct TemplateDeps {
    hash_map: HashMap<PathBuf, Vec<PathBuf>>,
}

pub const PATH: &'static str = ".noten/template-deps.toml";

impl TemplateDeps {
    pub fn open() -> io::Result<Self> {
        use toml;
        use std::fs::File;
        use std::io::prelude::*;

        let mut f = try!(File::open(PATH));
        let mut s = String::new();
        try!(f.read_to_string(&mut s));
        let mut parser = toml::Parser::new(&s);
        let table = parser.parse().unwrap();
        let mut hash_map = HashMap::new();
        for (k, v) in &table {
            let k: PathBuf = k.into();
            hash_map.insert(k.clone(), Vec::new());
            if let toml::Value::Array(ref vec) = *v {
                for p in vec {
                    hash_map.get_mut(&k).unwrap().push(p.as_str().unwrap().into());
                }
            } else {
                panic!("Array expected");
            }
        }
        Ok(TemplateDeps { hash_map: hash_map })
    }
    pub fn clear_deps(&mut self, template_path: PathBuf) {
        if let Some(entry) = self.hash_map.get_mut(&template_path) {
            entry.clear();
        }
    }
    pub fn add_dep(&mut self, template_path: PathBuf, dep_path: PathBuf) {
        use std::collections::hash_map::Entry;
        debug!("Added dep: {:?} => {:?}", template_path, dep_path);

        match self.hash_map.entry(template_path) {
            Entry::Occupied(mut en) => en.get_mut().push(dep_path),
            Entry::Vacant(place) => {
                place.insert(vec![dep_path]);
            }
        }
    }
    pub fn save(&self) -> io::Result<()> {
        use toml;
        use std::fs::File;
        use std::io::prelude::*;

        let mut table = toml::Table::new();
        for (k, v) in &self.hash_map {
            let tp = k.to_string_lossy().into_owned();
            let mut array = toml::Array::new();
            for p in v {
                let dp = p.to_string_lossy().into_owned();
                array.push(toml::Value::String(dp));
            }
            table.insert(tp, toml::Value::Array(array));
        }
        let str = toml::encode_str(&table);
        let mut f = try!(File::create(PATH));
        f.write_all(str.as_bytes())
    }
}
