# alpacahack-tools
AlapacaHackの問題ディレクトリを作成するコマンド。現状、Daily AlpacaHackにのみ対応している。

## おおまかな使い方
初めて使う際は
```bash
alpacahack-tools config --workspace /home/username/ctf/alpacahack
```
のようにしてワークスペースを指定する。その後、
```bash
alpacahack-tools new
```
で起動。
```
問題ページのurl> 
```
と出てくるので、問題ページのURL(例えば `https://alpacahack.com/daily/challenges/a-fact-of-ctf` など)を入力する。すると、ワークスペースに`a-fact-of-CTF`のようなディレクトリが作成され、その中に`a-fact-of-CTF.tar.gz`ファイルが展開される。（添付ファイルが`.tar.gz`でないパターンは、そのまま配置される。）
その後、問題ディレクトリをVSCodeで開く。

## 各サブコマンドの仕様
### alpacahack-tools new
```bash
alpacahack-tools new --url https://alpacahack.com/daily/challenges/alpaca-nation
```
のようにして`alpaca-nation`のデータのダウンロードができる。また、`--today`が入ってる場合は今日のプロジェクトを作成することを優先する。