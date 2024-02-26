use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub async fn tee(
    mut input: impl AsyncRead + std::marker::Unpin,
    mut output_1: impl AsyncWrite + std::marker::Unpin,
    mut output_2: impl AsyncWrite + std::marker::Unpin,
) -> std::io::Result<()> {
    let mut buf = [0u8; 1024];
    loop {
        let num_read = input.read(&mut buf).await?;
        if num_read == 0 {
            break;
        }

        let buf = &buf[..num_read];
        output_1.write_all(buf).await?;
        output_2.write_all(buf).await?;
    }

    Ok(())
}
