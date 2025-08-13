Import-Module Pester

Describe "nxsh basic" {
  It "prints hello" {
    $p = Start-Process -FilePath "$PSScriptRoot/../../target/release/nxsh.exe" -ArgumentList '-c','echo hello' -NoNewWindow -PassThru -RedirectStandardOutput out.txt -Wait
    $p.ExitCode | Should -Be 0
    (Get-Content out.txt -Raw) | Should -Match 'hello'
  }

  It "logstats --json outputs JSON" {
    $nx = "$PSScriptRoot/../../target/release/nxsh.exe"
    $p = Start-Process -FilePath $nx -ArgumentList '-c','logstats --json' -NoNewWindow -PassThru -RedirectStandardOutput out2.txt -Wait
    $p.ExitCode | Should -Be 0
    $json = Get-Content out2.txt -Raw
    $json.Trim().StartsWith('{') | Should -BeTrue
  }
}


