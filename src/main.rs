use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    process,
};

use anyhow::{Context, Result};
use reqwest::Url;

#[tokio::main]
async fn main() -> Result<()> {
    // ファイルのダウンロードURLを入力してもらう。
    let file_url = input_url().context("不正なURLです。")?;
    // AlpacaHackのディレクトリ名を取得する。
    let alpacahack_directory = get_alpacahack_directory()?;

    run(&file_url, &alpacahack_directory).await?;
    Ok(())
}

/// 指定した URL からファイルを非同期にダウンロードし、作業ディレクトリに展開して VS Code で開くユーティリティ関数。
///
/// # 引数
/// - `file_url`: ダウンロード対象のファイルを指す `Url`。
/// - `alpacahack_directory`: (未使用の場合は内部で決定される) ベースとなる作業ディレクトリの `Path`。
///
/// # 動作
/// 1. `file_url` からデータを非同期で取得する（`download` を呼ぶ）。
/// 2. URL からファイル名を抽出し、`alpacahack` 配下に問題用ディレクトリを作成する。
/// 3. ダウンロードしたデータを展開し、`memo.md` を作成する。
/// 4. `code` コマンドで作成したディレクトリを開く。
///
/// # エラー
/// 処理の各段階で失敗すると `Result::Err` を返す（各箇所でコンテキスト付きのエラーメッセージを追加）。
///
/// # Example
/// ```
/// run(
///     Url::parse("https://alpacahack-prod.s3.ap-northeast-1.amazonaws.com/0a2e166c-fe68-4617-83d2-1ff98a4e5812/a-fact-of-CTF.tar.gz")?,
///     home_dir.join("competitions").join("ctf").join("alpacahack")
/// )
/// ```
async fn run(file_url: &Url, alpacahack_directory: &Path) -> Result<()> {
    // ファイルをダウンロードする
    let downloaded_data = download(file_url)
        .await
        .context("ファイルのダウンロードに失敗しました。")?;
    println!("ダウンロードが完了しました。");

    // URLからファイル名を取得する。
    let downloaded_filename = get_filename(file_url).context("ファイル名の取得に失敗しました。")?;

    // 問題ディレクトリを作成する。
    let dir = create_directory(alpacahack_directory, &downloaded_filename)
        .context("問題ディレクトリの作成に失敗しました。")?;
    println!("問題ディレクトリを作成しました: {}", dir.display());

    // 問題ディレクトリの中にファイルを展開する。
    expand_file(&dir, &downloaded_filename, &downloaded_data)
        .context("ファイルの展開に失敗しました。")?;
    println!("ファイルの展開が完了しました。");

    // 問題ディレクトリにmemo.mdを作成する。
    let memo_path = dir.join("memo.md");
    File::create(&memo_path).context("memo.mdの作成に失敗しました。")?;
    println!("memo.mdを作成しました: {}", memo_path.display());

    // VSCodeでディレクトリを開く。
    open_vscode(&dir).context("VSCodeでディレクトリを開けませんでした。")?;
    println!("VSCodeでディレクトリを開きました。");

    Ok(())
}

/// URLを入力してもらう。
fn input_url() -> Result<Url> {
    print!("download url> ");
    io::stdout()
        .flush()
        .context("標準出力に失敗しました。")
        .unwrap();

    let mut url = String::new();
    io::stdin()
        .read_line(&mut url)
        .context("URLの入力に失敗しました")
        .unwrap();
    validate_domain(url.trim())
}

/// URLがalpacahack-prod.s3.ap-northeast-1.amazonaws.comのものであることを確認する。
fn validate_domain(url: &str) -> Result<Url> {
    let url = Url::parse(url)?;
    let domain = url
        .domain()
        .ok_or(anyhow::anyhow!("ドメイン名がありません。"))?;
    if domain == "alpacahack-prod.s3.ap-northeast-1.amazonaws.com" {
        Ok(url)
    } else {
        Err(anyhow::anyhow!("ドメイン名が正しくありません。"))
    }
}

/// ファイルをダウンロードする。
async fn download(url: &Url) -> Result<bytes::Bytes> {
    Ok(reqwest::get(url.as_str()).await?.bytes().await?)
}

/// URLからファイル名を取得する。
fn get_filename(url: &Url) -> Result<String> {
    let filename = url
        .path_segments()
        .ok_or(anyhow::anyhow!("URLのパスがありません。"))?
        .next_back()
        .ok_or(anyhow::anyhow!("URLのパスが空です。"))?;
    Ok(filename.to_owned())
}

/// 問題ディレクトリを作成する。
fn create_directory(alpacahack_directory: &Path, downloaded_filename: &str) -> Result<PathBuf> {
    // ファイル名の末尾の.tar.gzを削除したものをディレクトリ名とする。
    let dirname = downloaded_filename.to_string().replace(".tar.gz", "");
    let dir_path = alpacahack_directory.join(dirname);
    fs::create_dir_all(&dir_path)?;

    Ok(dir_path)
}

/// ディレクトリの中にファイルを展開する。
fn expand_file(dir: &Path, downloaded_filename: &str, downloaded_data: &[u8]) -> Result<()> {
    // ダウンロードしたファイルを保存する。
    let downloaded_file_path = dir.join(downloaded_filename);
    File::create(&downloaded_file_path)?.write_all(downloaded_data)?;
    // ダウンロードしたファイルがtar.gzの場合、解凍する。
    if downloaded_filename.ends_with(".tar.gz") {
        let tar_gz = File::open(&downloaded_file_path)?;
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);
        archive.unpack(dir)?;
    }
    // ダウンロードしたファイルを削除する。
    fs::remove_file(&downloaded_file_path)?;
    Ok(())
}

/// alpacahackディレクトリのパスを取得する。
fn get_alpacahack_directory() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or(anyhow::anyhow!(
        "ホームディレクトリが取得できませんでした。"
    ))?;
    let alpacahack_directory = home_dir.join("competitions").join("ctf").join("alpacahack");
    Ok(alpacahack_directory)
}

/// VSCodeで問題ディレクトリを開く。
fn open_vscode(problem_dir: &Path) -> Result<()> {
    process::Command::new("code")
        .arg(&problem_dir)
        .spawn()?
        .wait()?;
    Ok(())
}

#[cfg(test)]
mod daily_alpacahack_test {
    use super::*;
    use tempfile::tempdir;

    /// rootディレクトリにrelが存在することを確認する
    fn assert_exists(root: &Path, rel: &str) {
        assert!(root.join(rel).exists(), "{} does not exist", rel)
    }

    /// 問題名とファイル名が一致しているパターン
    #[tokio::test]
    async fn test_emojify_matching() {
        let _problem_url = Url::parse("https://alpacahack.com/daily/challenges/emojify").unwrap();
        let file_url = Url::parse("https://alpacahack-prod.s3.ap-northeast-1.amazonaws.com/5bad030b-a894-4111-900d-43332caf6bf6/emojify.tar.gz").unwrap();

        let dir = tempdir().unwrap();

        run(&file_url, dir.path()).await.unwrap();

        let expected = [
            "emojify/emojify/backend",
            "emojify/emojify/backend/index.js",
            "emojify/emojify/backend/package-lock.json",
            "emojify/emojify/backend/package.json",
            "emojify/emojify/frontend",
            "emojify/emojify/frontend/index.html",
            "emojify/emojify/frontend/index.js",
            "emojify/emojify/frontend/package-lock.json",
            "emojify/emojify/frontend/package.json",
            "emojify/emojify/secret",
            "emojify/emojify/secret/index.js",
            "emojify/emojify/secret/package-lock.json",
            "emojify/emojify/secret/package.json",
            "emojify/emojify/compose.yaml",
            "emojify/emojify/Dockerfile",
            "emojify/memo.md",
        ];
        for rel in expected {
            assert_exists(dir.path(), rel);
        }
    }

    /// 問題名とファイル名が一致していないパターン
    #[test]
    #[ignore]
    fn test_a_fact_of_ctf_mismatch() {
        let problem_url = "https://alpacahack.com/daily/challenges/a-fact-of-ctf";
        let file_url = "https://alpacahack-prod.s3.ap-northeast-1.amazonaws.com/0a2e166c-fe68-4617-83d2-1ff98a4e5812/a-fact-of-CTF.tar.gz";
    }

    /// ファイルが.tar.gzでないパターン
    #[test]
    #[ignore]
    fn test_non_tar_file() {
        let problem_url = "https://alpacahack.com/daily/challenges/read-assembly";
        let file_url = "https://alpacahack-prod.s3.ap-northeast-1.amazonaws.com/d8a7fbf5-1a2f-4398-ab06-bc1422cf33c6/asm.txt";
    }

    /// ファイルがないパターン
    #[test]
    #[ignore]
    fn test_no_file() {
        let problem_url = "https://alpacahack.com/daily/challenges/alpacahack-2100";
    }
}
