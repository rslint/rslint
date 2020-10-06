use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::try_init()?;

    let (service, messages) =
        LspService::new(|client| rslint_lsp::lsp::server::Server::new(client).unwrap());
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    Server::new(stdin, stdout)
        .interleave(messages)
        .serve(service)
        .await;

    Ok(())
}
