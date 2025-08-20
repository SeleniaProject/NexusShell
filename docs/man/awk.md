# awk

NexusShell の awk 実装は、一般的な awk の文法と機能を概ねカバーします。

主な対応項目:
- BEGIN/END ブロック
- パターン/アクション (`PATTERN { ACTIONS }`)
- 文字列・数値・論理式、三項演算子、+ - * / %、比較、連結、`in`、`~`/`!~`（拡張正規表現）
- 連想配列、配列インデックス `a[idx]`、`for (k in arr)`
- ユーザー定義関数（`function name(args){ ... }`）と `return`
- 組み込み関数（抜粋）: `length`, `substr`, `split`, `match`（RSTART/RLENGTH 設定）, `sprintf`, `int`, `sqrt`, `sin`, `cos`, `atan2`, `log`, `exp`, `tolower`, `toupper`, `rand`, `srand`, `system`
- フィールド/レコード: `FS`/`OFS`, `RS`/`ORS`, `NF`, `NR`, `FNR`, `FILENAME`
- フィールド参照 `$1`, `$0`、動的フィールド `$(expr)`
- `$n` 代入時の `$0`/`NF` 再構築
- 出力: `print`（引数間に `OFS`、末尾に `ORS`）/ `printf`（C 互換の幅/精度/フラグ）

制限/備考:
- 正規表現は `advanced-regex` フィーチャ有効時に強化（fancy-regex, aho-corasick）。
- 一部 GNU 拡張は未実装のものがあります。必要に応じて issue を立ててください。

## 使用例

- 単純な抽出と出力
```
awk '{ print $1, $3 }' input.txt
```

- 区切り文字の指定と再構築
```
awk -F, -v OFS='\t' '{ $2 = "Z"; print $0 }' data.csv
```

- BEGIN/END と集計
```
awk 'BEGIN{sum=0} {sum+= $2} END{printf("sum=%d\n", sum)}' file
```

- 正規表現と match()
```
awk '{ if (match($0, /[0-9]+/)) print RSTART, RLENGTH }' file
```

- ユーザー定義関数
```
awk 'function add(a,b){ return a+b } { print add($1,$2) }' file
```

## 互換性
- awk の数値/文字列の暗黙変換規則に準拠。
- `print`/`printf` は C 準拠のフォーマッタ互換を目指し、`-`, `+`, ` `, `#`, `0`、幅、精度をサポート。
- `match()` は 1-origin の開始位置を返し、`RSTART`/`RLENGTH` を設定します。

## トラブルシューティング
- 期待通りに `$0` が更新されない場合は `FS`/`OFS` 設定を確認してください。
- 複雑な正規表現を使う場合は `--features advanced-regex` でビルドしているか確認してください。
