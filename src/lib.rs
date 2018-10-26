use std::collections::BTreeMap;
use std::fmt;
use std::any::Any;

pub mod component;
pub mod library;
pub mod libraries;

struct Plan {
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
