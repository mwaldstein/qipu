class Qipu < Formula
  desc "Zettelkasten-inspired knowledge management CLI"
  homepage "https://github.com/mwaldstein/qipu"
  version "0.3.32"
  license "MIT"

  if Hardware::CPU.intel?
    url "https://github.com/mwaldstein/qipu/releases/download/v0.3.32/qipu-0.3.32-x86_64-apple-darwin.tar.gz"
    sha256 "49421ebfc2d0f87b73797dd06a0438b519b444dd6d679426a398b525568fe164"
  else
    url "https://github.com/mwaldstein/qipu/releases/download/v0.3.32/qipu-0.3.32-aarch64-apple-darwin.tar.gz"
    sha256 "4a9053549e3238705b9745fb0c99783e69bf2a8ff1f90b64feec0f9d12fe40c8"
  end

  def install
    bin.install "qipu"
  end

  test do
    system bin/"qipu", "--version"
  end
end
