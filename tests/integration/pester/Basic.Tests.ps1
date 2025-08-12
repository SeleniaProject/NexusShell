Import-Module Pester

Describe "nxsh basic" {
  It "prints hello" {
    $p = Start-Process -FilePath "$PSScriptRoot/../../target/release/nxsh.exe" -ArgumentList '-c','echo hello' -NoNewWindow -PassThru -RedirectStandardOutput out.txt -Wait
    $p.ExitCode | Should -Be 0
    (Get-Content out.txt -Raw) | Should -Match 'hello'
  }
}


