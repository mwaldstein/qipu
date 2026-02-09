class Qipu < Formula
  desc "Zettelkasten-inspired knowledge management CLI"
  homepage "https://github.com/mwaldstein/qipu"
  url "https://github.com/mwaldstein/qipu/archive/refs/tags/v0.3.19.tar.gz"
  sha256 "04c3f3ba4712b81252c29f4535e92e305efc8506da5f4f2499e0479eefa39d03"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system bin/"qipu", "--version"
  end
end
