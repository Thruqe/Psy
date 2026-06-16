pub use pseudocode_types::Value;

use std::collections::HashMap;

pub struct Environment {
    variables: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            variables: HashMap::new(),
        }
    }

    pub fn set(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Value {
        self.variables
            .get(name)
            .cloned()
            .unwrap_or(Value::Undefined)
    }

    pub fn declare_array(&mut self, name: &str, size: usize) {
        let mut array = Vec::with_capacity(size);
        for _ in 0..size {
            array.push(Value::Undefined);
        }
        self.variables.insert(name.to_string(), Value::Array(array));
    }

    pub fn set_array_element(&mut self, name: &str, index: usize, value: Value) {
        if let Some(Value::Array(array)) = self.variables.get_mut(name) {
            if index < array.len() {
                array[index] = value;
            }
        }
    }

    pub fn get_array_element(&self, name: &str, index: usize) -> Value {
        if let Some(Value::Array(array)) = self.variables.get(name) {
            if index < array.len() {
                return array[index].clone();
            }
        }
        Value::Undefined
    }

    pub fn print_state(&self) {
        println!("--- Variable State ---");
        for (name, value) in &self.variables {
            match value {
                Value::Number(n) => println!("{} = {}", name, n),
                Value::String(s) => println!("{} = \"{}\"", name, s),
                Value::Boolean(b) => println!("{} = {}", name, b),
                Value::Array(arr) => println!("{} = {:?}", name, arr),
                Value::Undefined => println!("{} = undefined", name),
            }
        }
        println!("----------------------");
    }
}
