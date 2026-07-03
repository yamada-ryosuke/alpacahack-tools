use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// alpacahack-toolsの設定
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    /// 各問題のプロジェクトを展開するワークスペースディレクトリのパス
    pub workspace: Option<PathBuf>,
}

impl Config {
    /// 設定を取得する。
    /// 設定ファイルが存在しない場合は設定ファイルを作成してから取得する。
    pub fn load() -> Result<Self> {
        file::load()
    }

    /// 設定を書き込む。
    /// 設定ファイルが存在しない場合は作成する。
    pub fn save(&self) -> Result<()> {
        file::save(self)
    }
}

/// 初期設定
impl Default for Config {
    fn default() -> Self {
        Self { workspace: None }
    }
}

/// 設定ファイルを扱うモジュール
/// configモジュール内ではファイル操作関連のクレートをuseしないような形にしたい。
mod file {
    use std::{
        fs::{self, File},
        io::Write,
        path::PathBuf,
    };

    use super::Config;
    use anyhow::{Context, Result};
    use directories::ProjectDirs;

    /// 設定を取得する。
    /// 設定ファイルが存在しない場合は設定ファイルを取得する。
    pub(super) fn load() -> Result<Config> {
        let config_path = get_config_file_path()?;
        // 設定ファイルがなければ作成する。
        if !get_config_file_path()?.exists() {
            Config::default().save()?;
            println!("新しく設定ファイルを作成しました。");
        }

        // 設定ファイルを読みだす。
        let config_data =
            fs::read_to_string(config_path).context("設定ファイルが読み込めませんでした。")?;

        // 設定ファイルをパースする。
        let config_data =
            toml::from_str(&config_data).context("設定ファイルがパースできませんでした。")?;

        Ok(config_data)
    }

    /// 設定を書き込む。
    /// 設定ファイルが存在しない場合は作成する。
    pub(super) fn save(config: &Config) -> Result<()> {
        let mut file = create_if_not_exists()?;

        let data = toml::to_string(config).context("設定データの文字列化に失敗しました。")?;
        file.write_all(data.as_bytes())
            .context("ファイルへの書き込みに失敗しました。")?;

        Ok(())
    }

    /// 設定ファイルを作成する。
    /// 存在する場合はファイルを書き込みモードで取得する。
    fn create_if_not_exists() -> Result<File> {
        let config_path = get_config_file_path()?;
        fs::create_dir_all(config_path.parent().unwrap())
            .context("設定ファイルを置くディレクトリの作成に失敗しました。")
            .unwrap();
        File::create(config_path).context("設定ファイルの作成または取得に失敗しました。")
    }

    /// 設定ファイルのパスを取得する。
    fn get_config_file_path() -> Result<PathBuf> {
        Ok(get_project_dirs()?.config_local_dir().join("config.toml"))
    }

    /// alpacahack-toolsのプロジェクトディレクトリを取得する。
    // 今後CLIの補完とかのためにユーザーデータとかを作るならそこでも使うので、その時は別の場所に移動した方がいい。
    // この関数を別の場所に移せたらget_config_file_path()の名前をget_path()にできそう。
    // 今のままだとproject_dirsと設定ファイルのあるディレクトリが混ざりそうなので、今はできない。
    fn get_project_dirs() -> Result<ProjectDirs> {
        ProjectDirs::from("", "", "alpacahack-tools")
            .context("alpacahack-toolsのプロジェクトディレクトリを取得できませんでした。")
    }
}
