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



