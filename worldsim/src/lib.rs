#![feature(euclidean_division, duration_float, trait_alias, bind_by_move_pattern_guards, test)]

extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate test;

pub mod job;
pub mod regionmanager;
pub mod region;
pub mod server;
pub mod lodstore;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
