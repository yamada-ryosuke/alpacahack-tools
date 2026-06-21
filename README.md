# alpacahack-new
AlapacaHackの問題ディレクトリを作成するコマンド。
```bash
alpacaha-new
```
で起動。
```
問題ページのurl> 
```
と出てくるので、問題ページのURL(例えば `https://alpacahack.com/daily/challenges/a-fact-of-ctf` など)を入力すると、`~/competitions/ctf/alpacahack`に`a-fact-of-CTF`のようなディレクトリが作成され、その中に`a-fact-of-CTF.tar.gz`ファイルが展開される。`.tar.gz`でないパターンも対応済み。
その後、問題ディレクトリをVSCodeで開く。