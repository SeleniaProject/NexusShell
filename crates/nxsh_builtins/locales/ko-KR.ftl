# NexusShell 내장 명령어 - 한국어 현지화

# 공통 메시지
error-file-not-found = 파일을 찾을 수 없습니다: {$filename}
error-permission-denied = 권한이 거부되었습니다: {$filename}
error-invalid-option = 잘못된 옵션: {$option}
error-missing-argument = 옵션에 인수가 없습니다: {$option}
error-invalid-argument = 잘못된 인수: {$argument}
error-directory-not-found = 디렉토리를 찾을 수 없습니다: {$dirname}
error-not-a-directory = 디렉토리가 아닙니다: {$path}
error-not-a-file = 파일이 아닙니다: {$path}
error-operation-failed = 작업이 실패했습니다: {$operation}
error-io-error = I/O 오류: {$message}

# cat 명령어
cat-help-usage = 사용법: cat [옵션]... [파일]...
cat-help-description = 파일을 표준 출력으로 연결합니다.
cat-version = cat (NexusShell) 1.0.0

# ls 명령어
ls-help-usage = 사용법: ls [옵션]... [파일]...
ls-help-description = 파일 정보를 나열합니다(기본값은 현재 디렉토리).
ls-permission-read = 읽기
ls-permission-write = 쓰기
ls-permission-execute = 실행
ls-type-directory = 디렉토리
ls-type-file = 일반 파일
ls-type-symlink = 심볼릭 링크

# grep 명령어
grep-help-usage = 사용법: grep [옵션]... 패턴 [파일]...
grep-help-description = 각 파일에서 패턴을 검색합니다.
grep-matches-found = {$count}개의 일치 항목을 찾았습니다
grep-no-matches = 일치하는 항목이 없습니다

# ps 명령어
ps-help-usage = 사용법: ps [옵션]...
ps-help-description = 실행 중인 프로세스 정보를 표시합니다.
ps-header-pid = PID
ps-header-user = 사용자
ps-header-command = 명령어

# ping 명령어
ping-help-usage = 사용법: ping [옵션]... 호스트
ping-help-description = 네트워크 호스트에 ICMP ECHO_REQUEST를 보냅니다.
ping-statistics = --- {$host} ping 통계 ---
ping-packets-transmitted = {$transmitted}개 패킷 전송됨
ping-packets-received = {$received}개 수신됨
ping-packet-loss = {$loss}% 패킷 손실

# 공통 파일 작업
file-exists = 파일이 존재합니다: {$filename}
file-not-exists = 파일이 존재하지 않습니다: {$filename}
operation-cancelled = 작업이 취소되었습니다
operation-completed = 작업이 성공적으로 완료되었습니다 