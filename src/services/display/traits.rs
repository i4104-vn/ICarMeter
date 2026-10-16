// Display Backend abstraction interface

pub trait DisplayBackend {
    fn is_open(&self) -> bool;
    fn present(&mut self, buffer: &[u32]) -> Result<(), String>;
    fn backend_name(&self) -> &str;
}
