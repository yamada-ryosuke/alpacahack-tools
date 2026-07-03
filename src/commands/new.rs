/// 問題ページから問題の情報を取得する機能のモジュール
mod fetch;
/// 問題プロジェクトを作成する機能のモジュール
mod project;

use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    process,
};

use anyhow::{Context, Result};

use crate::{cli::NewArgs, config::Config, prelude::*};

/// サブコマンド`new`の動作
pub fn run(args: NewArgs) {
    // 設定ファイルからワークスペースのパスを取り出す。
    // ワークスペースの設定が存在しない場合はエラー。
    let config = Config::load().expect("設定を取得できませんでした。");
    let workspace = get_workspace(&config).unwrap();

    // コマンドライン引数から問題のURLを取得する。
    // コマンドライン引数で指定されていない場合は対話的に入力してもらう。
    // その後、URLをバリデートする。
    let challenge_url = get_challenge_url(&args)
        .context("URLの取得に失敗しました。")
        .unwrap();

    let challenge_dir = setup_challenge_project(&challenge_url, &workspace).unwrap();

    // VSCodeでディレクトリを開く。
    open_vscode(&challenge_dir)
        .context("VSCodeでディレクトリを開けませんでした。")
        .unwrap();
    println!("VSCodeでディレクトリを開きました。");
}

/// 設定からワークスペースのパスを取得する。
/// ワークスペースの設定が存在しない場合はエラー。
fn get_workspace(config: &Config) -> Result<PathBuf> {
    match &config.workspace {
        Some(workspace) => Ok(workspace.clone()),
        None => Err(anyhow::anyhow!(
            "ワークスペースのパスが設定されていません。\n`alpacahack-tools config set --workspace <workspace-full-path>`を実行してください。"
        )),
    }
}

/// コマンドライン引数から問題のURLを取得する。
/// コマンドライン引数で指定されていない場合は対話的に入力してもらう。
fn get_challenge_url(args: &NewArgs) -> Result<AlpacaHackUrl> {
    let url = match &args.url {
        Some(url) => url.to_string(),
        None => input_url().context("入出力に失敗しました。").unwrap(),
    };
    AlpacaHackUrl::new(url.trim()).context("不正なURLです。")
}

/// 問題ページのURLを入力してもらう。
fn input_url() -> Result<String> {
    print!("問題ページのurl> ");
    io::stdout()
        .flush()
        .context("標準出力に失敗しました。")
        .unwrap();

    let mut url = String::new();
    io::stdin()
        .read_line(&mut url)
        .context("URLの入力に失敗しました")
        .unwrap();
    Ok(url)
}

/// 指定した URL から問題データを取得し、作業ディレクトリに問題プロジェクトを作成する。
///
/// # 引数
/// - `file_url`: ダウンロード対象のファイルを指す `Url`。
/// - `workspace`: ワークスペースの `Path`。
///
/// # 動作
/// 1. `file_url` から問題データを非同期で取得する（`fetch::fetch_problem_data` を呼ぶ）。
/// 2. 取得した問題情報をもとに、`alpacahack` 配下に問題用プロジェクトを作成する。
///
/// # 返り値
/// 作成した問題プロジェクトのディレクトリパス。
fn setup_challenge_project(challenge_url: &AlpacaHackUrl, workspace: &Path) -> Result<PathBuf> {
    // 問題情報を取得する。
    let challenge_info = fetch::fetch_challenge_data(challenge_url)?;
    println!("問題情報を取得しました");
    println!("問題タイトル: {}", challenge_info.meta.title);

    // 問題プロジェクトを作成する。
    let challenge_dir = project::create_project(workspace, challenge_info)?;
    println!("問題プロジェクトの作成が完了しました。");

    Ok(challenge_dir)
}

/// VSCodeで問題ディレクトリを開く。
fn open_vscode(challenge_dir: &Path) -> Result<()> {
    process::Command::new("code")
        .arg(challenge_dir)
        .spawn()?
        .wait()?;
    Ok(())
}

#[cfg(test)]
mod daily_alpacahack_test {
    use std::{fs, io::Read};

    use super::*;
    use chrono::NaiveDate;
    use tempfile::tempdir;

    /// 期待されるディレクトリ構成を表す構造体
    enum FsEntry {
        File(String),
        Directory(String, Vec<FsEntry>),
    }

    /// ディレクトリ構成が正しいことを確かめる関数
    fn assert_directory_structure(root: &Path, expected: &FsEntry) {
        match expected {
            FsEntry::File(name) => {
                let path = root.join(name);
                assert!(
                    path.is_file(),
                    "{}はファイルではありません。",
                    path.display()
                );
            }
            FsEntry::Directory(name, children) => {
                let dir = root.join(name);
                assert!(
                    dir.is_dir(),
                    "{}はディレクトリではありません。",
                    dir.display()
                );
                for child in children {
                    assert_directory_structure(&dir, child);
                }
            }
        }
    }

    /// challenge.tomlの中身が正しいことのテスト
    fn assert_challenge_toml(root: &Path, expected_challenge_toml: ChallengeMeta) {
        let mut challenge_toml = String::new();
        fs::File::open(
            root.join(&expected_challenge_toml.project_name)
                .join("challenge.toml"),
        )
        .unwrap()
        .read_to_string(&mut challenge_toml)
        .unwrap();
        let challenge_toml = toml::from_str::<ChallengeMeta>(&challenge_toml).unwrap();
        assert_eq!(challenge_toml, expected_challenge_toml);
    }

    /// 問題タイトルとファイル名が一致しているパターン
    #[test]
    fn test_emojify_matching() {
        let challenge_url =
            AlpacaHackUrl::new("https://alpacahack.com/daily/challenges/emojify").unwrap();

        let dir = tempdir().unwrap();

        setup_challenge_project(&challenge_url, dir.path()).unwrap();

        use FsEntry::*;
        let expected_directory = Directory(
            "emojify".into(),
            vec![
                File("note.md".into()),
                File("challenge.toml".into()),
                Directory(
                    "emojify".into(),
                    vec![
                        Directory(
                            "backend".into(),
                            vec![
                                File("index.js".into()),
                                File("package-lock.json".into()),
                                File("package.json".into()),
                            ],
                        ),
                        Directory(
                            "frontend".into(),
                            vec![
                                File("index.js".into()),
                                File("index.html".into()),
                                File("package-lock.json".into()),
                                File("package.json".into()),
                            ],
                        ),
                        Directory(
                            "secret".into(),
                            vec![
                                File("index.js".into()),
                                File("package-lock.json".into()),
                                File("package.json".into()),
                            ],
                        ),
                        File("compose.yaml".into()),
                        File("Dockerfile".into()),
                    ],
                ),
            ],
        );

        assert_directory_structure(dir.path(), &expected_directory);

        // challenge.tomlの中身が正しいことのテスト
        assert_challenge_toml(
            dir.path(),
            ChallengeMeta {
                url: challenge_url,
                released_at: NaiveDate::from_ymd_opt(2025, 12, 3).unwrap(),
                title: "Emojify".into(),
                project_name: "emojify".into(),
            },
        );
    }

    /// 問題タイトルとファイル名が一致していないパターン
    #[test]
    fn test_a_fact_of_ctf_mismatch() {
        let challenge_url =
            AlpacaHackUrl::new("https://alpacahack.com/daily/challenges/a-fact-of-ctf").unwrap();

        let dir = tempdir().unwrap();

        setup_challenge_project(&challenge_url, dir.path()).unwrap();

        use FsEntry::*;
        let expected_directory = Directory(
            "a-fact-of-ctf".into(),
            vec![
                File("note.md".into()),
                File("challenge.toml".into()),
                Directory(
                    "a-fact-of-CTF".into(),
                    vec![File("chall.py".into()), File("output.txt".into())],
                ),
            ],
        );
        assert_directory_structure(dir.path(), &expected_directory);

        // challenge.tomlの中身が正しいことのテスト
        assert_challenge_toml(
            dir.path(),
            ChallengeMeta {
                url: challenge_url,
                released_at: NaiveDate::from_ymd_opt(2025, 12, 2).unwrap(),
                title: "a fact of CTF".into(),
                project_name: "a-fact-of-ctf".into(),
            },
        );
    }

    /// ファイルが.tar.gzでないパターン
    #[test]
    fn test_non_tar_file() {
        let challenge_url =
            AlpacaHackUrl::new("https://alpacahack.com/daily/challenges/read-assembly").unwrap();

        let dir = tempdir().unwrap();

        setup_challenge_project(&challenge_url, dir.path()).unwrap();

        use FsEntry::*;
        let expected_directory = Directory(
            "read-assembly".into(),
            vec![
                File("note.md".into()),
                File("challenge.toml".into()),
                File("asm.txt".into()),
            ],
        );
        assert_directory_structure(dir.path(), &expected_directory);

        // challenge.tomlの中身が正しいことのテスト
        assert_challenge_toml(
            dir.path(),
            ChallengeMeta {
                url: challenge_url,
                released_at: NaiveDate::from_ymd_opt(2025, 12, 10).unwrap(),
                title: "Read Assembly".into(),
                project_name: "read-assembly".into(),
            },
        );
    }

    /// ファイルがないパターン
    #[test]
    fn test_no_file() {
        let challenge_url =
            AlpacaHackUrl::new("https://alpacahack.com/daily/challenges/alpacahack-2100").unwrap();

        let dir = tempdir().unwrap();

        setup_challenge_project(&challenge_url, dir.path()).unwrap();

        use FsEntry::*;
        let expected_directory = Directory(
            "alpacahack-2100".into(),
            vec![File("note.md".into()), File("challenge.toml".into())],
        );
        assert_directory_structure(dir.path(), &expected_directory);

        // challenge.tomlの中身が正しいことのテスト
        assert_challenge_toml(
            dir.path(),
            ChallengeMeta {
                url: challenge_url,
                released_at: NaiveDate::from_ymd_opt(2025, 12, 1).unwrap(),
                title: "AlpacaHack 2100".into(),
                project_name: "alpacahack-2100".into(),
            },
        );
    }
}
