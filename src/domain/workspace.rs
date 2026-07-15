use anyhow::Error;

use crate::{config::Config, prelude::*};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

/// ワークスペースを表す構造体。
///
/// ワークスペースは、プロジェクトや問題ファイルを格納するためのルートディレクトリを管理する。
#[derive(Debug, Clone)]
pub struct Workspace {
    path: PathBuf,
}

impl Workspace {
    /// # 概要
    /// `path` を基に `Workspace` を作成する。
    ///
    /// # 動作
    /// - 指定した `path` のディレクトリが存在しない場合、作成する。
    pub fn new(path: &Path) -> Result<Self> {
        fs::create_dir_all(path)?;

        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    /// ワークスペースのパスを取得する。
    pub fn get_path(&self) -> &Path {
        &self.path
    }

    /// # 概要
    /// 指定した `ChallengeInfo` を基に新しい問題プロジェクトを作成する。
    ///
    /// # 引数
    /// - `challenge_info`: 作成するプロジェクトのメタ情報と添付ファイル。
    ///
    /// # 動作
    /// 1. ワークスペース配下にプロジェクト用ディレクトリを作成する。
    /// 2. 添付ファイルを展開し、`note.md` と `challenge.toml` を生成してプロジェクトを初期化する。
    ///
    /// # 戻り値
    /// 成功すると作成した `Project` を返す。ディレクトリが既に存在する場合、エラーを返す。
    pub fn create_project(&self, challenge_info: &ChallengeInfo) -> Result<Project> {
        self.daily().create_project(challenge_info)
    }

    /// `daily` ディレクトリを返す。
    /// ただし、ディレクトリの存在は保証されていない。
    fn daily(&self) -> Daily {
        let daily_path = self.path.join("challenges").join("daily");
        Daily { path: daily_path }
    }

    /// 名前でプロジェクトを検索する。
    ///
    /// # 戻り値
    /// 見つかった `Project` を返す。
    fn search_project_by_project_name(&self, project_name: &str) -> Result<Option<Project>> {
        self.daily().search_project_by_project_name(&project_name)
    }

    /// URLでプロジェクトを検索する。
    ///
    /// # 戻り値
    /// 見つかった `Project` を返す。
    pub fn search_project_by_url(&self, url: &ChallengeUrl) -> Result<Option<Project>> {
        let project_name = url.project_name();
        self.daily().search_project_by_project_name(&project_name)
    }
}

impl TryFrom<&Config> for Workspace {
    type Error = Error;

    /// 設定からワークスペースを取得する。
    /// ワークスペースの設定が存在しない場合、エラーを返す。
    fn try_from(config: &Config) -> Result<Self> {
        let path = config.workspace.clone().ok_or(anyhow::anyhow!("ワークスペースのパスが設定されていません。\n`alpacahack-tools config set --workspace <workspace-full-path>`を実行してください。"))?;

        Workspace::new(&path)
    }
}

/// Daily AlpacaHack のディレクトリ
/// <workspace>/challenge/daily内のプロジェクトを配置するディレクトリの指定の責務を負う。
#[derive(Debug, Clone)]
struct Daily {
    path: PathBuf,
}

impl Daily {
    /// <workspace>/challenge/daily内にプロジェクトを作成する。
    fn create_project(&self, challenge_info: &ChallengeInfo) -> Result<Project> {
        let project_path = self.project_path(&challenge_info.meta);
        // 既に同名のディレクトリが存在していないことを確認する。
        // 「指定したパスが妥当であるか？」を確認する処理なので、Project側ではなくDaily側の責務。
        if project_path.exists() {
            anyhow::bail!(
                "`{}`のディレクトリは既に存在しています。",
                challenge_info.meta.project_name
            );
        }

        Project::create(&project_path, challenge_info)
    }

    /// `ChallengeMeta` を基にプロジェクトのパスを取得する。
    fn project_path(&self, challenge_meta: &ChallengeMeta) -> PathBuf {
        let released_at = &challenge_meta.released_at;
        let project_name = &challenge_meta.project_name;

        self.path
            .join(released_at.format("%Y-%m").to_string())
            .join(project_name)
    }

    /// daily内のプロジェクトの一覧を返す。
    fn projects(&self) -> Result<Vec<Project>> {
        let mut projects = Vec::new();

        // dailyディレクトリが存在しない場合はプロジェクトが存在しないと考える。
        if !self.path.exists() {
            return Ok(projects);
        }

        for month_dir in fs::read_dir(&self.path)? {
            let month_path = month_dir?.path();
            if !month_path.is_dir() {
                continue;
            }
            for project_dir in fs::read_dir(&month_path)? {
                let project_path = project_dir?.path();
                if !project_path.is_dir() {
                    continue;
                }
                projects.push(Project { path: project_path });
            }
        }
        Ok(projects)
    }

    /// 名前でプロジェクトを検索する。
    ///
    /// # 戻り値
    /// 見つかった `Project` を返す。
    fn search_project_by_project_name(&self, name: &str) -> Result<Option<Project>> {
        let match_projects: Vec<Project> = self
            .projects()?
            .into_iter()
            .filter(|project| project.name() == name)
            .collect();

        // 2つ以上同名のプロジェクトがあったらエラー
        match match_projects.len() {
            0 => Ok(None),
            1 => Ok(Some(match_projects[0].clone())),
            other => {
                let mut message = "同名のプロジェクトが2つ以上存在します。\n".to_string();
                for project in &match_projects {
                    message = format!("{}{}\n", &message, project.get_path().display());
                }
                Err(anyhow::anyhow!(message))
            }
        }
    }
}

/// 問題プロジェクト
/// この型の値が存在することは、対応するプロジェクトがファイルシステム上に実際に存在する
/// ことを保証する責務を持つ。また、プロジェクト内でのファイル配置（note.md や
/// challenge.toml、添付ファイルの展開先など）に関する扱いを一手に引き受ける責務を負い、
/// プロジェクトの初期化や構成に関する操作を提供する。
///
#[derive(Debug, Clone)]
pub struct Project {
    /// プロジェクトのパス
    path: PathBuf,
}

impl Project {
    /// プロジェクトのパスを取得する。
    pub fn get_path(&self) -> &Path {
        &self.path
    }

    /// プロジェクト名を取得する。
    pub fn name(&self) -> String {
        self.path.file_name().unwrap().to_string_lossy().to_string()
    }

    /// プロジェクトを作成する。
    fn create(path: &Path, challenge_info: &ChallengeInfo) -> Result<Self> {
        // ディレクトリを作成する。
        fs::create_dir_all(path).context("問題ディレクトリの作成に失敗しました。")?;
        println!("問題ディレクトリを作成しました: {}", path.display());

        let project = Project {
            path: path.to_path_buf(),
        };
        // チャレンジの情報を基にプロジェクトを初期化する。
        project.init(challenge_info)?;

        Ok(project)
    }

    /// `ChallengeInfo` を基にプロジェクトを初期化する。
    fn init(&self, challenge_info: &ChallengeInfo) -> Result<()> {
        let meta = &challenge_info.meta;
        let attached = &challenge_info.attached;

        // 問題に添付されているファイルを展開する。
        if let Some(data) = attached {
            self.add_attached_file(data)
                .context("ファイルの展開に失敗しました。")?;
            println!("ファイルの展開が完了しました。");
        }

        // note.mdを作成する。
        let note_path = self.add_note(&meta.title)?;
        println!("note.mdを作成しました: {}", note_path.display());

        // challenge.tomlを作成する。
        self.add_challenge_toml(meta)?;

        Ok(())
    }

    /// ディレクトリの中にダウンロードしたファイルを展開する。
    fn add_attached_file(&self, downloaded_file: &ChallengeFile) -> Result<PathBuf> {
        // ダウンロードしたファイルを保存する。
        let downloaded_file_path = self.path.join(&downloaded_file.name);
        File::create(&downloaded_file_path)?.write_all(&downloaded_file.data)?;

        // ダウンロードしたファイルがtar.gzの場合、解凍する。
        if downloaded_file.name.ends_with(".tar.gz") {
            let tar_gz = File::open(&downloaded_file_path)?;
            let tar = flate2::read::GzDecoder::new(tar_gz);
            let mut archive = tar::Archive::new(tar);
            archive.unpack(&self.path)?;
            // ダウンロードしたファイルを削除する。
            fs::remove_file(&downloaded_file_path)?;
        }

        Ok(downloaded_file_path)
    }

    /// 解法などを書くためのnote.mdを作成する。
    fn add_note(&self, title: &str) -> Result<PathBuf> {
        // 問題ディレクトリにnote.mdを作成する。
        let note_path = self.path.join("note.md");
        let mut note_file = File::create(&note_path).context("note.mdの作成に失敗しました。")?;

        note_file.write_all(format!("# {}\n\n", title).as_bytes())?;
        note_file.write_all("## 解法\n\n".as_bytes())?;
        note_file.write_all("## 学び\n\n".as_bytes())?;

        Ok(note_path)
    }

    /// 問題情報を記述した challenge.toml を作成する。
    fn add_challenge_toml(&self, meta: &ChallengeMeta) -> Result<PathBuf> {
        let toml_path = self.path.join("challenge.toml");
        let data = toml::to_string_pretty(&meta)?;

        let mut toml_file =
            File::create(&toml_path).context("challenge.tomlの作成に失敗しました。")?;
        toml_file.write_all(data.as_bytes())?;

        Ok(toml_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use chrono::NaiveDate;
    use reqwest::Url;
    use tempfile::TempDir;

    /// テスト用の ChallengeUrl を作成する
    fn create_test_url() -> ChallengeUrl {
        ChallengeUrl::new("https://alpacahack.com/daily/challenges/test-challenge")
            .expect("テスト用URLの作成に失敗しました")
    }

    /// テスト用の ChallengeMeta を作成する
    fn create_test_meta() -> ChallengeMeta {
        ChallengeMeta {
            url: create_test_url(),
            released_at: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            title: "テスト問題".to_string(),
            project_name: "test-challenge".to_string(),
        }
    }

    /// テスト用の ChallengeInfo を作成する（ファイルなし）
    fn create_test_challenge_info() -> ChallengeInfo {
        ChallengeInfo {
            meta: create_test_meta(),
            attached: None,
        }
    }

    /// テスト用の ChallengeInfo を作成する（ファイル付き）
    fn create_test_challenge_info_with_file(filename: &str, data: &[u8]) -> ChallengeInfo {
        ChallengeInfo {
            meta: create_test_meta(),
            attached: Some(ChallengeFile {
                _url: Url::parse("https://example.com/file.txt").unwrap(),
                name: filename.to_string(),
                data: Bytes::copy_from_slice(data),
            }),
        }
    }

    #[test]
    fn test_workspace_new_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().join("workspace");

        assert!(!workspace_path.exists());

        let workspace = Workspace::new(&workspace_path).unwrap();

        assert!(workspace_path.exists());
        assert_eq!(workspace.get_path(), workspace_path);
    }

    #[test]
    fn test_workspace_new_with_existing_directory() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();

        let workspace = Workspace::new(workspace_path).unwrap();

        assert_eq!(workspace.get_path(), workspace_path);
    }

    #[test]
    fn test_workspace_get_path() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();

        let workspace = Workspace::new(workspace_path).unwrap();
        let retrieved_path = workspace.get_path();

        assert_eq!(retrieved_path, workspace_path);
    }

    #[test]
    fn test_create_project_without_attached_file() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::new(temp_dir.path()).unwrap();

        let challenge_info = create_test_challenge_info();
        let project = workspace.create_project(&challenge_info).unwrap();

        // プロジェクトディレクトリが作成されたことを確認
        assert!(project.get_path().exists());

        // note.md が作成されたことを確認
        let note_path = project.get_path().join("note.md");
        assert!(note_path.exists());
        let note_content = std::fs::read_to_string(&note_path).unwrap();
        assert!(note_content.contains("# テスト問題"));
        assert!(note_content.contains("## 解法"));
        assert!(note_content.contains("## 学び"));

        // challenge.toml が作成されたことを確認
        let toml_path = project.get_path().join("challenge.toml");
        assert!(toml_path.exists());
        let toml_content = std::fs::read_to_string(&toml_path).unwrap();
        assert!(toml_content.contains("test-challenge"));
    }

    #[test]
    fn test_create_project_with_attached_file() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::new(temp_dir.path()).unwrap();

        let file_content = b"test file content";
        let challenge_info = create_test_challenge_info_with_file("test.txt", file_content);
        let project = workspace.create_project(&challenge_info).unwrap();

        // ファイルが展開されたことを確認
        let file_path = project.get_path().join("test.txt");
        assert!(file_path.exists());
        let saved_content = std::fs::read(&file_path).unwrap();
        assert_eq!(saved_content, file_content);
    }

    #[test]
    fn test_create_project_duplicate_fails() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::new(temp_dir.path()).unwrap();

        let challenge_info = create_test_challenge_info();
        let _project = workspace.create_project(&challenge_info).unwrap();

        // 同じプロジェクトを作成しようとするとエラーになる
        let result = workspace.create_project(&challenge_info);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("既に存在しています")
        );
    }

    #[test]
    fn test_search_project_by_project_name_found() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::new(temp_dir.path()).unwrap();

        let challenge_info = create_test_challenge_info();
        let _created_project = workspace.create_project(&challenge_info).unwrap();

        // 作成したプロジェクトを検索
        let found_project = workspace
            .search_project_by_project_name("test-challenge")
            .unwrap();

        assert!(found_project.is_some());
        let project = found_project.unwrap();
        assert!(project.get_path().exists());
    }

    #[test]
    fn test_search_project_by_project_name_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::new(temp_dir.path()).unwrap();

        // 存在しないプロジェクトを検索
        let found_project = workspace
            .search_project_by_project_name("non-existent")
            .unwrap();

        assert!(found_project.is_none());
    }

    #[test]
    fn test_search_project_by_url() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::new(temp_dir.path()).unwrap();

        let challenge_info = create_test_challenge_info();
        let _created_project = workspace.create_project(&challenge_info).unwrap();

        // URLでプロジェクトを検索
        let url = create_test_url();
        let found_project = workspace.search_project_by_url(&url).unwrap();

        assert!(found_project.is_some());
    }

    #[test]
    fn test_project_path_structure() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::new(temp_dir.path()).unwrap();

        let challenge_info = create_test_challenge_info();
        let project = workspace.create_project(&challenge_info).unwrap();

        // プロジェクトパスが正しい構造を持つことを確認
        // <workspace>/challenges/daily/2024-01/test-challenge
        let project_path = project.get_path();
        assert!(
            project_path
                .to_string_lossy()
                .contains("challenges/daily/2024-01/test-challenge")
        );
    }

    #[test]
    fn test_project_get_path() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::new(temp_dir.path()).unwrap();

        let challenge_info = create_test_challenge_info();
        let project = workspace.create_project(&challenge_info).unwrap();

        let path = project.get_path();
        assert!(path.exists());
        assert!(path.is_dir());
    }

    #[test]
    fn test_note_md_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::new(temp_dir.path()).unwrap();

        let title = "重要な問題";
        let mut meta = create_test_meta();
        meta.title = title.to_string();

        let challenge_info = ChallengeInfo {
            meta,
            attached: None,
        };

        let project = workspace.create_project(&challenge_info).unwrap();

        let note_path = project.get_path().join("note.md");
        assert!(note_path.exists());

        let content = std::fs::read_to_string(&note_path).unwrap();
        assert_eq!(content, format!("# {}\n\n## 解法\n\n## 学び\n\n", title));
    }

    #[test]
    fn test_challenge_toml_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::new(temp_dir.path()).unwrap();

        let challenge_info = create_test_challenge_info();
        let project = workspace.create_project(&challenge_info).unwrap();

        let toml_path = project.get_path().join("challenge.toml");
        assert!(toml_path.exists());

        let content = std::fs::read_to_string(&toml_path).unwrap();
        // CHallengeMeta を正しくシリアライズしたことを確認
        assert!(content.contains("test-challenge"));
        assert!(content.contains("テスト問題"));
        assert!(content.contains("2024-01-15"));
    }

    #[test]
    fn test_multiple_projects_in_different_months() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = Workspace::new(temp_dir.path()).unwrap();

        // 1月のプロジェクト
        let challenge_info_jan = create_test_challenge_info();
        let project_jan = workspace.create_project(&challenge_info_jan).unwrap();

        // 2月のプロジェクト
        let mut challenge_info_feb = create_test_challenge_info();
        challenge_info_feb.meta.released_at = NaiveDate::from_ymd_opt(2024, 2, 15).unwrap();
        challenge_info_feb.meta.project_name = "test-challenge-feb".to_string();
        let _url_feb =
            ChallengeUrl::new("https://alpacahack.com/daily/challenges/test-challenge-feb")
                .expect("テスト用URLの作成に失敗しました");
        challenge_info_feb.meta.url = _url_feb;

        let project_feb = workspace.create_project(&challenge_info_feb).unwrap();

        // 異なるディレクトリに作成されたことを確認
        assert_ne!(project_jan.get_path(), project_feb.get_path());
        assert!(project_jan.get_path().to_string_lossy().contains("2024-01"));
        assert!(project_feb.get_path().to_string_lossy().contains("2024-02"));

        // 両方とも存在することを確認
        assert!(project_jan.get_path().exists());
        assert!(project_feb.get_path().exists());
    }
}
