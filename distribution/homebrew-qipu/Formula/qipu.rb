class Qipu < Formula
  desc "Zettelkasten-inspired knowledge management CLI"
  homepage "https://github.com/mwaldstein/qipu"
  version "0.3.32"
  license "MIT"

  if Hardware::CPU.intel?
    url "https://github.com/mwaldstein/qipu/releases/download/v0.3.32/qipu-0.3.32-x86_64-apple-darwin.tar.gz"
    sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  else
    url "https://github.com/mwaldstein/qipu/releases/download/v0.3.32/qipu-0.3.32-aarch64-apple-darwin.tar.gz"
    sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  end

  def install
    bin.install "qipu"
  end

  test do
    system bin/"qipu", "--version"
  end
end
