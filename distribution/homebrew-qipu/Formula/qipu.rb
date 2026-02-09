class Qipu < Formula
  desc "Zettelkasten-inspired knowledge management CLI"
  homepage "https://github.com/mwaldstein/qipu"
  url "https://github.com/mwaldstein/qipu/archive/refs/tags/v0.3.6.tar.gz"
  sha256 "c044c42d01b1fa21490d4a8d5c2e47cfdb9de321ce08b7cdb626b67fe5c118fd"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    system bin/"qipu", "--version"
  end
end
