use crate::prelude::*;
use std::{
    fs::{self, File}, io::Write, path::{Path, PathBuf},
};

pub struct Workspace {
    path: PathBuf,
}

impl Workspace {
    /// プロジェクトのパスを取得する
    pub fn get_path(&self) -> &Path{
        &self.path
    }

    /// pathをパスとするWorkspaceを取得する。
    /// もしpathのディレクトリが存在しない場合はディレクトリを作成する。
    pub fn new(path: &Path) -> Result<Self> {
        fs::create_dir_all(path)?;

        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    fn get_daily(&self) -> Result<Daily> {
        let daily_path = self.path.join("daily");
        fs::create_dir_all(&daily_path)?;
        Ok(Daily { path: daily_path })
    }

    fn find_project_by_name() -> Vec<Project> {
        todo!()
    }

    /// 与えたデータを基に問題プロジェクトを作成する。
    ///
    /// # 引数
    /// - `workspace`: ワークスペースの `Path`。
    /// - `challenge_info`: 問題の情報の。
    ///
    /// # 動作
    /// 1. `file_url` から問題データを非同期で取得する（`fetch::fetch_problem_data` を呼ぶ）。
    /// 2. 取得した問題情報をもとに、`alpacahack` 配下に問題用プロジェクトを作成する。
    ///
    /// # 返り値
    /// 作成した問題プロジェクトのディレクトリパス。
    pub fn create_project(&self, challenge_info: &ChallengeInfo) -> Result<Project> {
        let meta = &challenge_info.meta;
        let project_path = self.get_project_path(meta);
        // 既に同名のディレクトリが存在していないことを確認する。
        if project_path.exists() {
            return Err(anyhow::anyhow!(
                "`{}`のディレクトリは既に存在しています。",
                meta.project_name
            ));
        }

        fs::create_dir_all(&project_path).context("問題ディレクトリの作成に失敗しました。")?;

        println!(
            "問題ディレクトリを作成しました: {}",
            project_path.display()
        );

        let project = Project { path: project_path };

        project.init(challenge_info);

        Ok(project)
    }

    /// プロジェクトのパスを取得する
    fn get_project_path(&self, challenge_meta: &ChallengeMeta) -> PathBuf {
        let released_at = &challenge_meta.released_at;
        let project_name = &challenge_meta.project_name;
        self.path
            .join("challenges")
            .join("daily")
            .join(released_at.format("%Y-%m").to_string())
            .join(project_name)
    }
}

/// Daily AlpacaHackのディレクトリ
struct Daily {
    path: PathBuf,
}

impl Daily {
    fn find_project_by_project_name(name: &str) -> Option<Project> {
        todo!();
    }
}

/// 問題プロジェクト
pub struct Project {
    /// プロジェクトのパス
    path: PathBuf,
}

impl Project {
    /// プロジェクトのパスを取得する
    pub fn get_path(&self) -> &Path{
        &self.path
    }

    /// 初期化ファイルを基にプロジェクトを初期化する。
    fn init(&self, challenge_info: &ChallengeInfo) -> Result<()> {
        let meta = &challenge_info.meta;
        let attached = &challenge_info.attached;

        // 問題に添付されているファイルを展開する。
        if let Some(data) = attached {
            self.expand_file( data).context("ファイルの展開に失敗しました。")?;
            println!("ファイルの展開が完了しました。");
        }

        // note.mdを作成する。
        let note_path = self.create_note(&meta.title)?;
        println!("note.mdを作成しました: {}", note_path.display());

        self.create_challenge_toml(meta)?;

        Ok(())
    }

    /// ディレクトリの中にダウンロードしたファイルを展開する。
    fn expand_file(&self, downloaded_file: &ChallengeFile) -> Result<PathBuf> {
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

    /// 解法などを書くためのnote.mdを作成する
    fn create_note(&self, title: &str) -> Result<PathBuf> {
        // 問題ディレクトリにnote.mdを作成する。
        let note_path = self.path.join("note.md");
        let mut note_file = File::create(&note_path).context("note.mdの作成に失敗しました。")?;

        note_file.write_all(format!("# {}\n\n", title).as_bytes())?;
        note_file.write_all("## 解法\n\n".as_bytes())?;
        note_file.write_all("## 学び\n\n".as_bytes())?;

        Ok(note_path)
    }

    /// 問題の情報などが書かれたchallenge.tomlを作成する
    fn create_challenge_toml(&self, meta: &ChallengeMeta) -> Result<PathBuf> {
        let toml_path = self.path.join("challenge.toml");
        let data = toml::to_string_pretty(&meta)?;

        let mut toml_file =
            File::create(&toml_path).context("challenge.tomlの作成に失敗しました。")?;
        toml_file.write_all(data.as_bytes())?;

        Ok(toml_path)
    }
}
