class Qipu < Formula
  desc "Zettelkasten-inspired knowledge management CLI"
  homepage "https://github.com/mwaldstein/qipu"
  url "https://github.com/mwaldstein/qipu/archive/refs/tags/v0.3.8.tar.gz"
  sha256 "75b0ed7e01fd21fb87c28984677e47a76c70f039fed05d8cab6e491d81b10040"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system bin/"qipu", "--version"
  end
end
