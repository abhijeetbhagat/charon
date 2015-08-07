pub trait Statement{
    fn generate_code(&self)->Vec<String>;
}