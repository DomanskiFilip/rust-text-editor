// main error module implements a wrapper that catches and handles errors
pub struct MainErrorWrapper;

impl MainErrorWrapper {
    pub fn handle_error<T, E>(result: Result<T, E>) where E: std::fmt::Debug {
        // for now just print error message
        if let Err(ref error) = result {
            eprintln!("Error: {error:#?}");
        }
    }
}


