use anyhow::{Context, Result};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use crate::challenge_info::{ChallengeData, ChallengeInfo};

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
    let challenge_dir = create_directory(alpacahack_dir, &challenge_info.meta.title_with_kebab)
        .context("問題ディレクトリの作成に失敗しました。")?;
    println!(
        "問題ディレクトリを作成しました: {}",
        challenge_dir.display()
    );

    // 問題に添付されているファイルを展開する。
    if let Some(data) = challenge_info.data {
        expand_file(&challenge_dir, data).context("ファイルの展開に失敗しました。")?;
        println!("ファイルの展開が完了しました。");
    }

    // note.mdを作成する。
    let note_path = create_note(&challenge_dir, &challenge_info.meta.title_with_space)?;
    println!("note.mdを作成しました: {}", note_path.display());

    Ok(challenge_dir)
}

/// 問題プロジェクトのディレクトリを作成する。
fn create_directory(alpacahack_dir: &Path, downloaded_filename: &str) -> Result<PathBuf> {
    // ファイル名の末尾の.tar.gzを削除したものをディレクトリ名とする。
    let dirname = downloaded_filename.to_string().replace(".tar.gz", "");
    let dir_path = alpacahack_dir.join(dirname);
    fs::create_dir_all(&dir_path)?;

    Ok(dir_path)
}

/// ディレクトリの中にダウンロードしたファイルを展開する。
fn expand_file(problem_dir: &Path, downloaded_file: ChallengeData) -> Result<()> {
    // ダウンロードしたファイルを保存する。
    let downloaded_file_path = problem_dir.join(&downloaded_file.name);
    File::create(&downloaded_file_path)?.write_all(&downloaded_file.data)?;
    // ダウンロードしたファイルがtar.gzの場合、解凍する。
    if downloaded_file.name.ends_with(".tar.gz") {
        let tar_gz = File::open(&downloaded_file_path)?;
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);
        archive.unpack(problem_dir)?;
        // ダウンロードしたファイルを削除する。
        fs::remove_file(&downloaded_file_path)?;
    }
    Ok(())
}

/// note.mdを作成する
fn create_note(challenge_dir: &Path, title: &str) -> Result<PathBuf> {
    // 問題ディレクトリにnote.mdを作成する。
    let note_path = challenge_dir.join("note.md");
    let mut note_file = File::create(&note_path).context("note.mdの作成に失敗しました。")?;

    note_file.write_all(format!("# {}\n\n", title).as_bytes())?;
    note_file.write_all("## 解法\n\n".as_bytes())?;
    note_file.write_all("## 学び\n\n".as_bytes())?;
    Ok(note_path)
}
