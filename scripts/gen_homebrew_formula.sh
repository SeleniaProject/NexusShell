#!/usr/bin/env bash
# Generates a Homebrew formula file for the given version and tarball.
# Usage: gen_homebrew_formula.sh <version> <artifact.tar.gz>
set -euo pipefail
VERSION="$1"
ARTIFACT="$2"
FORMULA_NAME="nexusshell"

# Compute SHA256 checksum of the artifact
SHA256=$(shasum -a 256 "$ARTIFACT" | awk '{print $1}')

cat > ${FORMULA_NAME}.rb <<EOF
class Nexusshell < Formula
  desc "NexusShell next-generation CLI shell"
  homepage "https://github.com/SeleniaProject/NexusShell"
  version "${VERSION}"
  url "https://github.com/SeleniaProject/NexusShell/releases/download/${VERSION}/${ARTIFACT}"
  sha256 "${SHA256}"
  license "Proprietary"

  def install
    bin.install "nxsh"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/nxsh --version")
  end
end
EOF

echo "Generated Homebrew formula: ${FORMULA_NAME}.rb" 