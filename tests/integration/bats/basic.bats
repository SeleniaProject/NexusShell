#!/usr/bin/env bats

@test "nxsh -c echo works" {
  run ./target/release/nxsh -c "echo hello"
  [ "$status" -eq 0 ]
  [[ "$output" == *"hello"* ]]
}


