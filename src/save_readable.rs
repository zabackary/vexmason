use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub async fn save_readable(
    mut input: impl AsyncRead + std::marker::Unpin,
    mut output_1: impl AsyncWrite + std::marker::Unpin,
) -> std::io::Result<Vec<u8>> {
    let mut buf = [0u8; 1024];
    let mut output = Vec::new();
    loop {
        let num_read = input.read(&mut buf).await?;
        if num_read == 0 {
            break;
        }

        let buf = &buf[..num_read];
        output_1.write_all(buf).await?;
        output.write_all(buf).await?;
    }

    Ok(output)
}
