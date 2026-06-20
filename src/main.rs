/// 問題ページから問題の情報を取得する機能のモジュール
mod fetch;
/// 問題の情報を持つための構造体
mod problem_info;
/// 問題プロジェクトを作成する機能のモジュール
mod project;

use std::{
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

    let problem_dir = setup_problem_project(&file_url, &alpacahack_directory).await?;

    // VSCodeでディレクトリを開く。
    open_vscode(&problem_dir).context("VSCodeでディレクトリを開けませんでした。")?;
    println!("VSCodeでディレクトリを開きました。");

    Ok(())
}

/// 指定した URL から問題データを取得し、作業ディレクトリに問題プロジェクトを作成する。
///
/// # 引数
/// - `file_url`: ダウンロード対象のファイルを指す `Url`。
/// - `alpacahack_directory`: ベースとなる作業ディレクトリの `Path`。
///
/// # 動作
/// 1. `file_url` から問題データを非同期で取得する（`fetch::fetch_problem_data` を呼ぶ）。
/// 2. 取得した問題情報をもとに、`alpacahack` 配下に問題用プロジェクトを作成する。
///
/// # 返り値
/// 作成した問題プロジェクトのディレクトリパス。
async fn setup_problem_project(file_url: &Url, alpacahack_directory: &Path) -> Result<PathBuf> {
    // 問題情報を取得する。
    let problem_info = fetch::fetch_problem_data(file_url).await?;
    println!("問題情報を取得しました");

    // 問題プロジェクトを作成する。
    let problem_dir = project::create_project(alpacahack_directory, problem_info)?;
    println!("問題プロジェクトを作成しました。");

    Ok(problem_dir)
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
    let url = validate_domain(url.trim());
    url
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
        .arg(problem_dir)
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

        setup_problem_project(&file_url, dir.path()).await.unwrap();

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
