use std::{
  io::{self},
  pin::Pin,
  task::Poll,
};

use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use tokio::io::{AsyncBufRead, AsyncRead};
use tokio_tar::ArchiveBuilder;

use crate::plugins::{
  Distro,
  sys_deploy::utils::{postinst, prepare_disk, write_fstab},
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Compression {
  Zstd,
  Gzip,
  Bzip2,
  Xz,
  Lzma,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Config {
  pub url: String,
  pub compression: Option<Compression>,
  #[serde(flatten)]
  pub common: super::CommonConfig,
}

impl crate::plugins::Plugin for Config {
  type Context = crate::plugins::Global;

  async fn invoke(&self, ctx: &Self::Context) -> anyhow::Result<()> {
    log::info!("System Deployer with config: {self:?}; globals: {ctx:?}");

    let (use_mdev, use_udev) = match ctx.distro_hint.as_ref() {
      Some(Distro::Alpine) => (true, false), // Alpine uses mdev
      Some(Distro::Arch) | Some(Distro::Debian) | Some(Distro::Fedora) | Some(Distro::Ubuntu) => (false, true), // Arch, Debian, Fedora, and Ubuntu use udev
      _ => {
        log::warn!(
          "Unknown distro hint: {:?}, defaulting to no mdev or udev",
          ctx.distro_hint
        );
        (false, false)
      } // Unknown or unspecified distro, default to no mdev or udev
    };
    prepare_disk(
      self.common.disk.as_str(),
      use_mdev,
      use_udev,
      self.common.mount.as_str(),
    )
    .await?;
    extract_tarball(self.url.as_str(), self.common.mount.as_str(), &self.compression).await?;
    write_fstab(self.common.disk.as_str(), self.common.mount.as_str()).await?;
    postinst(self.common.mount.as_str(), &self.common.distro).await?;
    Ok(())
  }
}

struct HttpStream {
  _inner: Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Unpin>,
  _buf: Option<Bytes>,
}

impl HttpStream {
  async fn fetch(url: &str) -> anyhow::Result<Self> {
    let client = reqwest::Client::new();
    log::info!("Fetching stream from URL: {url}");
    let stream = client.get(url).send().await?.bytes_stream();
    Ok(HttpStream {
      _inner: Box::new(stream),
      _buf: None,
    })
  }
}

impl AsyncRead for HttpStream {
  fn poll_read(
    mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>, buf: &mut tokio::io::ReadBuf<'_>,
  ) -> Poll<io::Result<()>> {
    if let Some(mut r_buf) = self._buf.take() {
      let max_len = buf.remaining();
      if r_buf.len() > max_len {
        self._buf = Some(r_buf.split_off(max_len));
      }
      buf.put_slice(&r_buf);
      return Poll::Ready(Ok(()));
    }

    match self._inner.poll_next_unpin(cx) {
      Poll::Ready(Some(Ok(mut r_buf))) => {
        let max_len = buf.remaining();
        if r_buf.len() > max_len {
          self._buf = Some(r_buf.split_off(max_len));
        }
        buf.put_slice(&r_buf);
        Poll::Ready(Ok(()))
      }
      Poll::Ready(Some(Err(e))) => Poll::Ready(Err(io::Error::other(e))),
      Poll::Ready(None) => Poll::Ready(Ok(())),
      Poll::Pending => Poll::Pending,
    }
  }
}

enum MaybeRemoteStream {
  Local(tokio::fs::File),
  Remote(HttpStream),
}

impl AsyncRead for MaybeRemoteStream {
  fn poll_read(
    self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>, buf: &mut tokio::io::ReadBuf<'_>,
  ) -> std::task::Poll<std::io::Result<()>> {
    let this = self.get_mut();
    match this {
      MaybeRemoteStream::Local(file) => {
        let f = Pin::new(file);
        f.poll_read(cx, buf)
      }
      MaybeRemoteStream::Remote(stream) => {
        let s = Pin::new(stream);
        s.poll_read(cx, buf)
      }
    }
  }
}

enum MaybeCompressedStream<S> {
  Plain(S),
  Zstd(async_compression::tokio::bufread::ZstdDecoder<S>),
  Gzip(async_compression::tokio::bufread::GzipDecoder<S>),
  Bzip2(async_compression::tokio::bufread::BzDecoder<S>),
  Xz(async_compression::tokio::bufread::XzDecoder<S>),
  Lzma(async_compression::tokio::bufread::LzmaDecoder<S>),
}

impl<S: AsyncBufRead + Unpin> AsyncRead for MaybeCompressedStream<S> {
  fn poll_read(
    self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>, buf: &mut tokio::io::ReadBuf<'_>,
  ) -> std::task::Poll<std::io::Result<()>> {
    let this = self.get_mut();
    match this {
      MaybeCompressedStream::Plain(stream) => {
        let s = Pin::new(stream);
        s.poll_read(cx, buf)
      }
      MaybeCompressedStream::Zstd(decoder) => {
        let d = Pin::new(decoder);
        d.poll_read(cx, buf)
      }
      MaybeCompressedStream::Gzip(decoder) => {
        let d = Pin::new(decoder);
        d.poll_read(cx, buf)
      }
      MaybeCompressedStream::Bzip2(decoder) => {
        let d = Pin::new(decoder);
        d.poll_read(cx, buf)
      }
      MaybeCompressedStream::Xz(decoder) => {
        let d = Pin::new(decoder);
        d.poll_read(cx, buf)
      }
      MaybeCompressedStream::Lzma(decoder) => {
        let d = Pin::new(decoder);
        d.poll_read(cx, buf)
      }
    }
  }
}

pub(crate) async fn extract_tarball(url: &str, dest: &str, compression: &Option<Compression>) -> anyhow::Result<()> {
  let backing_stream = if url.starts_with("http://") || url.starts_with("https://") {
    log::info!("Downloading tarball from {url} to {dest}");
    let http_stream = HttpStream::fetch(url).await?;
    MaybeRemoteStream::Remote(http_stream)
  } else {
    log::info!("Opening local tarball file: {url}");
    let file = tokio::fs::File::open(url).await?;
    MaybeRemoteStream::Local(file)
  };
  let compressed_stream = match compression {
    Some(Compression::Zstd) => {
      let zstd_decoder = async_compression::tokio::bufread::ZstdDecoder::new(tokio::io::BufReader::new(backing_stream));
      MaybeCompressedStream::Zstd(zstd_decoder)
    }
    Some(Compression::Gzip) => {
      let gzip_decoder = async_compression::tokio::bufread::GzipDecoder::new(tokio::io::BufReader::new(backing_stream));
      MaybeCompressedStream::Gzip(gzip_decoder)
    }
    Some(Compression::Bzip2) => {
      let bzip2_decoder = async_compression::tokio::bufread::BzDecoder::new(tokio::io::BufReader::new(backing_stream));
      MaybeCompressedStream::Bzip2(bzip2_decoder)
    }
    Some(Compression::Xz) => {
      let xz_decoder = async_compression::tokio::bufread::XzDecoder::new(tokio::io::BufReader::new(backing_stream));
      MaybeCompressedStream::Xz(xz_decoder)
    }
    Some(Compression::Lzma) => {
      let lzma_decoder = async_compression::tokio::bufread::LzmaDecoder::new(tokio::io::BufReader::new(backing_stream));
      MaybeCompressedStream::Lzma(lzma_decoder)
    }
    None => MaybeCompressedStream::Plain(tokio::io::BufReader::new(backing_stream)),
  };
  let mut archive = ArchiveBuilder::new(compressed_stream)
    .set_allow_external_symlinks(true)
    .set_ignore_zeros(false)
    .set_overwrite(true)
    .set_preserve_mtime(true)
    .set_preserve_permissions(true)
    .set_preserve_ownerships(true)
    .set_unpack_xattrs(true)
    .build();
  archive.unpack(dest).await?;
  Ok(())
}
