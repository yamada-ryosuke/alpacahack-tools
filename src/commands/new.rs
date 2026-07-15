use crate::{cli::NewArgs, config::Config, editor, prelude::*};

/// 新しい問題プロジェクトを作成してエディタで開くメイン処理。
///
/// 処理の流れ:
/// 1. 設定ファイルからワークスペースのパスを取得する（未設定の場合はエラー）。
/// 2. コマンドライン引数または対話入力から問題ページの URL を取得・検証する。
/// 3. 指定した URL から問題データを取得し、ワークスペース内に問題用ディレクトリと
///    プロジェクトファイルを作成する。
/// 4. 作成したディレクトリを VSCode で開く。
///
/// 引数:
/// - `args`: CLI から渡された New コマンドの引数。
///
/// エラーは内部で expect/unwrap により処理され、失敗時はメッセージを表示して終了する。
pub fn run(args: NewArgs) {
    // 設定ファイルからワークスペースのパスを取り出す。
    // ワークスペースの設定が存在しない場合はエラー。
    let config = Config::load().expect("設定を取得できませんでした。");
    let workspace = Workspace::try_from(&config).unwrap();

    // コマンドライン引数から問題のURLを取得する。
    // コマンドライン引数で指定されていない場合は対話的に入力してもらう。
    // その後、URLをバリデートする。
    let challenge_url = get_challenge_url(&args)
        .context("URLの取得に失敗しました。")
        .unwrap();

    let project = setup_challenge_project(&challenge_url, &workspace).unwrap();

    // VSCodeでディレクトリを開く。
    editor::open_with_vscode(&project)
        .context("VSCodeでディレクトリを開けませんでした。")
        .unwrap();
    println!("VSCodeでディレクトリを開きました。");
}

/// コマンドライン引数から問題のURLを取得する。
/// コマンドライン引数で指定されていない場合は対話的に入力してもらう。
fn get_challenge_url(args: &NewArgs) -> Result<ChallengeUrl> {
    let url = match &args.url {
        Some(url) => url.to_string(),
        None => input_url().context("入出力に失敗しました。").unwrap(),
    };
    ChallengeUrl::new(url.trim()).context("不正なURLです。")
}

/// 問題ページのURLを入力してもらう。
fn input_url() -> Result<String> {
    inquire::Text::new("問題ページのURL> ")
        .prompt()
        .context("URLの入力に失敗しました")
}

/// 指定した URL から問題データを取得し、作業ディレクトリに問題プロジェクトを作成する。
///
/// # 引数
/// - `file_url`: ダウンロード対象のファイルを指す `Url`。
/// - `workspace`: ワークスペースの `Path`。
///
/// # 動作
/// 1. `file_url` から問題データを非同期で取得する（`fetch::fetch_problem_data` を呼ぶ）。
/// 2. 取得した問題情報をもとに、ワークスペース 配下に問題用プロジェクトを作成する。
///
/// # 返り値
/// 作成した問題プロジェクトのディレクトリパス。
fn setup_challenge_project(challenge_url: &ChallengeUrl, workspace: &Workspace) -> Result<Project> {
    // 問題情報を取得する。
    let challenge_info = challenge_page::fetch(challenge_url)?;
    println!("問題情報を取得しました");
    println!("問題タイトル: {}", challenge_info.meta.title);

    // 問題プロジェクトを作成する。
    let project = workspace.create_project(&challenge_info)?;
    println!("問題プロジェクトの作成が完了しました。");

    Ok(project)
}

#[cfg(test)]
mod daily_alpacahack_test {
    use std::{fs, io::Read, path::Path};

    use super::*;
    use chrono::NaiveDate;
    use tempfile::tempdir;

    /// 期待されるディレクトリ構成を表す構造体
    enum FsEntry {
        File(String),
        Directory(String, Vec<FsEntry>),
    }

    /// ディレクトリ構成が正しいことを確かめる関数
    fn assert_directory_structure(challenge_parent_dir: &Path, expected: &FsEntry) {
        match expected {
            FsEntry::File(name) => {
                let path = challenge_parent_dir.join(name);
                assert!(path.exists(), "{}が存在しません。", path.display());
                assert!(
                    path.is_file(),
                    "{}はファイルではありません。",
                    path.display()
                );
            }
            FsEntry::Directory(name, children) => {
                let dir = challenge_parent_dir.join(name);
                assert!(dir.exists(), "{}が存在しません。", dir.display());
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
    fn assert_challenge_toml(challenge_dir: &Path, expected_challenge_toml: ChallengeMeta) {
        let mut challenge_toml = String::new();
        fs::File::open(challenge_dir.join("challenge.toml"))
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
            ChallengeUrl::new("https://alpacahack.com/daily/challenges/emojify").unwrap();

        let workspace = Workspace::new(tempdir().unwrap().path()).unwrap();

        setup_challenge_project(&challenge_url, &workspace).unwrap();

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

        // 問題ディレクトリ
        let challenge_dir = workspace
            .get_path()
            .join("challenges")
            .join("daily")
            .join("2025-12")
            .join("emojify");
        assert_directory_structure(challenge_dir.parent().unwrap(), &expected_directory);

        // challenge.tomlの中身が正しいことのテスト
        assert_challenge_toml(
            &challenge_dir,
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
            ChallengeUrl::new("https://alpacahack.com/daily/challenges/a-fact-of-ctf").unwrap();

        let workspace = Workspace::new(tempdir().unwrap().path()).unwrap();

        setup_challenge_project(&challenge_url, &workspace).unwrap();

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

        // 問題ディレクトリ
        let challenge_dir = workspace
            .get_path()
            .join("challenges")
            .join("daily")
            .join("2025-12")
            .join("a-fact-of-ctf");
        assert_directory_structure(challenge_dir.parent().unwrap(), &expected_directory);

        // challenge.tomlの中身が正しいことのテスト
        assert_challenge_toml(
            &challenge_dir,
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
            ChallengeUrl::new("https://alpacahack.com/daily/challenges/read-assembly").unwrap();

        let workspace = Workspace::new(tempdir().unwrap().path()).unwrap();

        setup_challenge_project(&challenge_url, &workspace).unwrap();

        use FsEntry::*;
        let expected_directory = Directory(
            "read-assembly".into(),
            vec![
                File("note.md".into()),
                File("challenge.toml".into()),
                File("asm.txt".into()),
            ],
        );

        // 問題ディレクトリ
        let challenge_dir = workspace
            .get_path()
            .join("challenges")
            .join("daily")
            .join("2025-12")
            .join("read-assembly");
        assert_directory_structure(challenge_dir.parent().unwrap(), &expected_directory);

        // challenge.tomlの中身が正しいことのテスト
        assert_challenge_toml(
            &challenge_dir,
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
            ChallengeUrl::new("https://alpacahack.com/daily/challenges/alpacahack-2100").unwrap();

        let workspace = Workspace::new(tempdir().unwrap().path()).unwrap();

        setup_challenge_project(&challenge_url, &workspace).unwrap();

        use FsEntry::*;
        let expected_directory = Directory(
            "alpacahack-2100".into(),
            vec![File("note.md".into()), File("challenge.toml".into())],
        );

        // 問題ディレクトリ
        let challenge_dir = workspace
            .get_path()
            .join("challenges")
            .join("daily")
            .join("2025-12")
            .join("alpacahack-2100");
        assert_directory_structure(challenge_dir.parent().unwrap(), &expected_directory);

        // challenge.tomlの中身が正しいことのテスト
        assert_challenge_toml(
            &challenge_dir,
            ChallengeMeta {
                url: challenge_url,
                released_at: NaiveDate::from_ymd_opt(2025, 12, 1).unwrap(),
                title: "AlpacaHack 2100".into(),
                project_name: "alpacahack-2100".into(),
            },
        );
    }
}
