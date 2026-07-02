use anyhow::{Context, Result};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use crate::prelude::*;

/// 与えたデータを基に問題プロジェクトを作成する。
///
/// # 引数
/// - `alpacahack_dir`: ベースとなる作業ディレクトリの `Path`。
/// - `challenge_info`: 問題の情報の。
///
/// # 動作
/// 1. `file_url` から問題データを非同期で取得する（`fetch::fetch_problem_data` を呼ぶ）。
/// 2. 取得した問題情報をもとに、`alpacahack` 配下に問題用プロジェクトを作成する。
///
/// # 返り値
/// 作成した問題プロジェクトのディレクトリパス。
pub(crate) fn create_project(
    alpacahack_dir: &Path,
    challenge_info: ChallengeInfo,
) -> Result<PathBuf> {
    // 問題ディレクトリを作成する。
    let challenge_dir = create_directory(alpacahack_dir, &challenge_info.meta.project_name)
        .context("問題ディレクトリの作成に失敗しました。")?;
    println!(
        "問題ディレクトリを作成しました: {}",
        challenge_dir.display()
    );

    // 問題に添付されているファイルを展開する。
    if let Some(data) = challenge_info.attached {
        expand_file(&challenge_dir, data).context("ファイルの展開に失敗しました。")?;
        println!("ファイルの展開が完了しました。");
    }

    // note.mdを作成する。
    let note_path = write_note(&challenge_dir, &challenge_info.meta.title)?;
    println!("note.mdを作成しました: {}", note_path.display());

    write_challenge_toml(&challenge_dir, &challenge_info.meta)?;

    Ok(challenge_dir)
}

/// 問題プロジェクトのディレクトリを作成する。
fn create_directory(alpacahack_dir: &Path, challenge_title: &str) -> Result<PathBuf> {
    let dir_path = alpacahack_dir.join(challenge_title);
    // 既に同名のディレクトリが存在していないことを確認する。
    if dir_path.exists() {
        return Err(anyhow::anyhow!(
            "`{}`のディレクトリは既に存在しています。",
            challenge_title
        ));
    }
    fs::create_dir_all(&dir_path)?;

    Ok(dir_path)
}

/// ディレクトリの中にダウンロードしたファイルを展開する。
fn expand_file(challenge_dir: &Path, downloaded_file: ChallengeFile) -> Result<PathBuf> {
    // ダウンロードしたファイルを保存する。
    let downloaded_file_path = challenge_dir.join(&downloaded_file.name);
    File::create(&downloaded_file_path)?.write_all(&downloaded_file.data)?;
    // ダウンロードしたファイルがtar.gzの場合、解凍する。
    if downloaded_file.name.ends_with(".tar.gz") {
        let tar_gz = File::open(&downloaded_file_path)?;
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);
        archive.unpack(challenge_dir)?;
        // ダウンロードしたファイルを削除する。
        fs::remove_file(&downloaded_file_path)?;
    }
    Ok(downloaded_file_path)
}

/// 解法などを書くためのnote.mdを作成する
fn write_note(challenge_dir: &Path, title: &str) -> Result<PathBuf> {
    // 問題ディレクトリにnote.mdを作成する。
    let note_path = challenge_dir.join("note.md");
    let mut note_file = File::create(&note_path).context("note.mdの作成に失敗しました。")?;

    note_file.write_all(format!("# {}\n\n", title).as_bytes())?;
    note_file.write_all("## 解法\n\n".as_bytes())?;
    note_file.write_all("## 学び\n\n".as_bytes())?;
    Ok(note_path)
}

/// 問題の情報などが書かれたchallenge.tomlを作成する
fn write_challenge_toml(challenge_dir: &Path, meta: &ChallengeMeta) -> Result<PathBuf> {
    let toml_path = challenge_dir.join("challenge.toml");
    let data = toml::to_string_pretty(&meta)?;

    let mut toml_file = File::create(&toml_path).context("challenge.tomlの作成に失敗しました。")?;
    toml_file.write_all(data.as_bytes())?;

    Ok(toml_path)
}
