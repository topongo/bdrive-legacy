#[derive(Debug)]
pub enum FileSuccess<T, S, O> {
    Yes(T),
    No(S, O)
}
