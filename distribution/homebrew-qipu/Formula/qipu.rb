class Qipu < Formula
  desc "Zettelkasten-inspired knowledge management CLI"
  homepage "https://github.com/mwaldstein/qipu"
  url "https://github.com/mwaldstein/qipu/archive/refs/tags/v0.3.7.tar.gz"
  sha256 "fa3e3ad00d761e0058f7ac4e8457999d9c139ddaf33401d7f9a6e1364e993936"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system bin/"qipu", "--version"
  end
end
