use std::fmt::{Display,Formatter};

#[derive(Debug)]
pub struct MyError {
    error: String,
}

impl MyError{
    pub fn from_string(error: &String) -> Self {
        MyError {error:error.clone() }
    }
    pub fn from_str(error: &str) -> Self {
        MyError {error:String::from(error) }
    }
}

impl Display for MyError{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result { 
        write!(f, "{}", self.error)
    }
}
impl std::error::Error for MyError{

}