// Toast notification enum
#[derive(Clone, Debug)]
pub enum Toaster {
    Success(String),
    Error(String),
    Warning(String),
    Info(String),
}
