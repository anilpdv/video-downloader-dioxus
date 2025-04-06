use dioxus::prelude::*;
use server_fn::error::NoCustomError;

/// Simple echo server function for testing
#[server(Echo)]
pub async fn echo(input: String) -> Result<String, ServerFnError<NoCustomError>> {
    Ok(input)
}
