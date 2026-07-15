# alpacahack-tools
AlapacaHackの問題ディレクトリを作成するコマンド。`.tar.gz`の解凍方法は一生覚えられないので。現状、Daily AlpacaHackにのみ対応している。

## おおまかな使い方
初めて使う際は
```bash
alpacahack-tools config set --workspace /home/username/ctf/alpacahack
```
のようにしてワークスペースをフルパスで指定する。その後、
```bash
alpacahack-tools new
```
で起動。
```
問題ページのURL> 
```
と出てくるので、問題ページのURL(例えば `https://alpacahack.com/daily/challenges/a-fact-of-ctf` など)を入力する。すると、ワークスペースに`a-fact-of-CTF`のようなディレクトリが作成され、その中に`a-fact-of-CTF.tar.gz`ファイルが展開される。（添付ファイルが`.tar.gz`でない場合は、ファイルがそのまま配置される。）
その後、問題ディレクトリをVSCodeで開く。

## 各サブコマンドの仕様

### alpacahack-tools new
```bash
alpacahack-tools new --url https://alpacahack.com/daily/challenges/alpaca-nation
```
のようにすると`alpaca-nation`のプロジェクトが作成される。`--url`を省略しても
```
問題ページのurl> 
```
と表示されるので、`https://alpacahack.com/daily/challenges/alpaca-nation`と入力すれば、同様に`alpaca-nation`のプロジェクトが作成される。その後VSCodeでプロジェクトを開く。

### alpacahack-tools new
```bash
alpacahack-tools open --url https://alpacahack.com/daily/challenges/alpaca-nation
```
のようにすると`alpaca-nation`のプロジェクトがVSCodeで開く。

### alpacahack-tools config
設定ファイル`/.config/alpacahack-tools/config.toml`を編集、表示できる。
```bash
alpacahack-tools config
```
で設定の一覧を表示できる。
```bash
alpacahack-tools config set
```
で設定を変更することができる。
```bash
alpacahack-tools config init
```
で設定を初期化できる。
