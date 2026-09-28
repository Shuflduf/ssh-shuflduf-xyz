pub trait RemoveFirst<T> {
    fn pop_first(&mut self) -> Option<T>;
}

impl<T> RemoveFirst<T> for Vec<T> {
    fn pop_first(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        Some(self.remove(0))
    }
}
