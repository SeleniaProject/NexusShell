# Preview QA Checklist (50 Items)

> This checklist must be completed before any Preview release of NexusShell is made available to testers.

## Functional Verification

1. [ ] Built-in `ls` correctly displays colorized output in ANSI/TrueColor.
2. [ ] Built-in `cd` updates `$PWD` and handles `cd -` edge-case.
3. [ ] Alias persistence via `~/.nxshrc` functions across restarts.
4. [ ] Parser properly tokenizes Unicode filenames (NFC/NFD).
5. [ ] Object pipeline converts JSON → table without data loss (`echo '{"a":1}' | select a`).
6. [ ] JIT disabled mode falls back to interpreter seamlessly.
7. [ ] Background job (`sleep 1 &`) is trackable via `jobs`.
8. [ ] `Ctrl+C` generates SIGINT, aborts running external process.
9. [ ] Completion engine suggests files, commands, and options contextually.
10. [ ] History encryption enabled when `NXSH_HISTORY_KEY` is set.

## Performance & Resource

11. [ ] Cold start ≤ 5 ms (average of 100 runs, `hyperfine`).
12. [ ] Steady-state RSS ≤ 15 MiB after executing `ls -R /usr`.
13. [ ] `grep -r TODO .` performance within 5 % of `ripgrep`.
14. [ ] p95 built-in execution latency < 2 ms (`nxsh_exec_latency_ms`).
15. [ ] No frame drops (>60 FPS) when executing continuous output (`yes | head -n 10000`).

## Cross-Platform

16. [ ] Linux x86-64 binary passes full test suite.
17. [ ] Linux AArch64 binary passes full test suite.
18. [ ] macOS (Apple Silicon) binary signs & notarizes correctly.
19. [ ] Windows 10+ build spawns external `cmd.exe /c dir`.
20. [ ] WASI build launches inside Wasmtime with minimal profile.

## Security & Compliance

21. [ ] `cargo audit` returns zero vulnerabilities.
22. [ ] SBOM (CycloneDX) included in artifacts.
23. [ ] Container image signed with Notary v2 and verified.
24. [ ] Secrets scanned via `trufflehog` report zero findings.
25. [ ] Binary reproducibility validated (`diffoscope`).

## Internationalization

26. [ ] Command output respects `LANG=ja_JP.UTF-8` for date/time.
27. [ ] UTF-8 prompt handles RTL scripts (Arabic/Hebrew) without misalignment.
28. [ ] Help system auto-selects language based on `LC_MESSAGES`.
29. [ ] `grep` correctly matches multibyte patterns in CJK.
30. [ ] Wide-character table rendering keeps columns aligned.

## UI / TUI

31. [ ] Splash screen fades in < 5 ms and respects terminal size.
32. [ ] Status bar updates every 100 ms without flicker.
33. [ ] Sidebar completion panel opens with `Tab` and is scrollable.
34. [ ] Toast overlay auto-dismisses after 3 s or on `Esc`.
35. [ ] High-contrast theme selectable via `nxsh --theme high-contrast`.

## Plugin System

36. [ ] WASM plugin registers built-in via `nx_plugin_register`.
37. [ ] Native crate plugin hot-reloaded without restarting shell.
38. [ ] Capability manifest denies FS write when not granted.
39. [ ] Plugin signature verified against trusted keyring.
40. [ ] Plugin crash is isolated; shell process continues.

## CI/CD Artifacts

41. [ ] Release tarballs include `nxsh`, `README`, `LICENSE`, `CHANGELOG`.
42. [ ] Homebrew formula installs binary to `/usr/local/bin/nxsh`.
43. [ ] Scoop manifest installs binary to `$env:USERPROFILE\scoop\apps\nxsh`.
44. [ ] Docker image tag `latest` points to newest release.
45. [ ] GitHub Actions matrix green across all targets.

## Documentation

46. [ ] `docs/CHANGELOG.md` updated with release entry.
47. [ ] Man pages generated under `docs/man/` for all built-ins.
48. [ ] DESIGN, SPEC, UI_DESIGN synced to implementation (no TODO markers).
49. [ ] README badges (build, coverage) reflect current status.
50. [ ] Preview release announcement draft prepared. 