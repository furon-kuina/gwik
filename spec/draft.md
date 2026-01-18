## 概要
Git Worktree を使いやすくする CLI

## 要件
- gwik open / close で新しいworktreeをブランチと一緒に作成/削除する
  - 既存のブランチに紐づけるときは何らかのオプションで指示する
- 自動で新しいworktreeのディレクトリに移動する
- fzf / peco との組み合わせで好きなworktreeに移動しやすくする
  - リポジトリ / Worktreeの2つの組み合わせがあるので、どうやったら使いやすい？
- .gitconfigに設定を記述する
- 設定に、worktree作成後、元のリポジトリから実行するコマンド（.envのコピーなど）、worktreeへの移動後に実行するコマンド（uv syncなど）を設定できるようにする
  - 元のリポジトリ、worktreeのリポジトリを何らかの変数でコマンドに埋め込めればOK


## 言語
- Rust