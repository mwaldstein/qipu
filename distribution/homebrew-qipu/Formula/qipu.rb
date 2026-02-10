class Qipu < Formula
  desc "Zettelkasten-inspired knowledge management CLI"
  homepage "https://github.com/mwaldstein/qipu"
  version "0.3.27"
  license "MIT"

  if Hardware::CPU.intel?
    url "https://github.com/mwaldstein/qipu/releases/download/v0.3.27/qipu-0.3.27-x86_64-apple-darwin.tar.gz"
    sha256 "0019dfc4b32d63c1392aa264aed2253c1e0c2fb09216f8e2cc269bbfb8bb49b5"
  else
    url "https://github.com/mwaldstein/qipu/releases/download/v0.3.27/qipu-0.3.27-aarch64-apple-darwin.tar.gz"
    sha256 "0019dfc4b32d63c1392aa264aed2253c1e0c2fb09216f8e2cc269bbfb8bb49b5"
  end

  def install
    bin.install "qipu"
  end

  test do
    system bin/"qipu", "--version"
  end
end
