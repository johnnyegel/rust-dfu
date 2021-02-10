
pub mod parse;

use std::process;

pub trait UnwrapOrDie<T> {
    fn unwrap_or_die(self, exit_code: i32, error: &str ) -> T;
}

impl<R, E> UnwrapOrDie<R> for Result<R, E> {
    fn unwrap_or_die(self, exit_code: i32, error: &str ) -> R {
        self.unwrap_or_else(|_| {
            eprintln!("Error: {}", error);
            process::exit(exit_code)
        })
    }
}
