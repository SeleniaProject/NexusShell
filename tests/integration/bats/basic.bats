#!/usr/bin/env bats

@test "nxsh -c echo works" {
  run ./target/release/nxsh -c "echo hello"
  [ "$status" -eq 0 ]
  [[ "$output" == *"hello"* ]]
}

@test "nxsh logstats --json outputs JSON" {
  run ./target/release/nxsh -c "logstats --json"
  [ "$status" -eq 0 ]
  [[ "$output" == \{* ]]
}

@test "zstd store-mode roundtrip works" {
  echo "abc123" > /tmp/nxsh_bats_zstd.txt
  run ./target/release/nxsh -c "zstd /tmp/nxsh_bats_zstd.txt"
  [ "$status" -eq 0 ]
  [ -f /tmp/nxsh_bats_zstd.txt.zst ]
  run ./target/release/nxsh -c "zstd -d -f /tmp/nxsh_bats_zstd.txt.zst"
  [ "$status" -eq 0 ]
  run cat /tmp/nxsh_bats_zstd.txt
  [[ "$output" == *"abc123"* ]]
}



