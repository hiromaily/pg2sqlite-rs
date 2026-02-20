# typed: false
# frozen_string_literal: true

# Homebrew formula for pg2sqlite
# Repository: https://github.com/hiromaily/homebrew-tap
class Pg2sqlite < Formula
  desc "PostgreSQL DDL to SQLite DDL schema converter written in Rust"
  homepage "https://github.com/hiromaily/pg2sqlite-rs"
  version "0.2.0"
  license "MIT"

  on_macos do
    on_intel do
      url "https://github.com/hiromaily/pg2sqlite-rs/releases/download/v#{version}/pg2sqlite-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_X86_64_DARWIN"
    end

    on_arm do
      url "https://github.com/hiromaily/pg2sqlite-rs/releases/download/v#{version}/pg2sqlite-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_AARCH64_DARWIN"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/hiromaily/pg2sqlite-rs/releases/download/v#{version}/pg2sqlite-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_X86_64_LINUX"
    end

    on_arm do
      url "https://github.com/hiromaily/pg2sqlite-rs/releases/download/v#{version}/pg2sqlite-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE_WITH_ACTUAL_SHA256_FOR_AARCH64_LINUX"
    end
  end

  def install
    bin.install "pg2sqlite"
  end

  test do
    # Create a test PostgreSQL DDL file
    (testpath/"test.sql").write <<~EOS
      CREATE TABLE users (
        id SERIAL PRIMARY KEY,
        name VARCHAR(100) NOT NULL
      );
    EOS

    # Run pg2sqlite and check it exits successfully
    system "#{bin}/pg2sqlite", testpath/"test.sql"
  end
end
