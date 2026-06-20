# alpacahack-new
AlapacaHackの問題ディレクトリを作成するコマンド。
```bash
alpacaha-new
```
で起動。
```
download url> 
```
と出てくるので、問題に添付されたファイルのダウンロードURL(例えば https://alpacahack-prod.s3.ap-northeast-1.amazonaws.com/0a2e166c-fe68-4617-83d2-1ff98a4e5812/a-fact-of-CTF.tar.gz など)を入力すると、`~/competitions/ctf/alpacahack`に`a-fact-of-CTF`のようなディレクトリが作成され、その中に`a-fact-of-CTF.tar.gz`ファイルが展開される。`.tar.gz`でないパターンも対応済み。