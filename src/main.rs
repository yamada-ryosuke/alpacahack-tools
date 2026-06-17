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
    let url = input_url()?;
    // ファイルをダウンロードする
    let downloaded_data = download(&url)
        .await
        .context("ファイルのダウンロードに失敗しました。")?;
    println!("ダウンロードが完了しました。");

    // URLからファイル名を取得する。
    let downloaded_filename = get_filename(&url).context("ファイル名の取得に失敗しました。")?;

    // alpacahackディレクトリのパスを取得する。
    let alpacahack_directory = get_alpacahack_directory()?;
    // 問題ディレクトリを作成する。
    let dir = create_directory(&alpacahack_directory, &downloaded_filename)
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
    process::Command::new("code")
        .arg(&dir)
        .spawn()
        .context("VSCodeでディレクトリを開けませんでした。")?;
    println!("VSCodeでディレクトリを開きました。");

    Ok(())
}

/// URLを入力してもらう。
fn input_url() -> Result<Url> {
    print!("download url> ");
    io::stdout().flush()?;

    let mut url = String::new();
    io::stdin().read_line(&mut url)?;
    validate_domain(url.trim())
}

/// ファイルをダウンロードする。
async fn download(url: &Url) -> Result<bytes::Bytes> {
    Ok(reqwest::get(url.as_str()).await?.bytes().await?)
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
