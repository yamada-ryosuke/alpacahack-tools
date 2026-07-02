use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use anyhow::{Context, Result};
use directories::{BaseDirs, ProjectDirs};
use serde::{Deserialize, Serialize};

/// alpacahack-newの設定
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    /// 各問題のプロジェクトを展開するワークスペースディレクトリのパス
    pub workspace: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            workspace: BaseDirs::new().unwrap().home_dir().join("alpacahack")
        }
    }
}

/// 設定ファイルが存在するか。
pub fn exists() -> bool {
    get_config_path().unwrap().exists()
}

/// 設定を取得する。
pub fn load() -> Result<Config> {
    let config_path = get_config_path()?;
    if !config_path.exists() {
        save(&Config::default())?;
        println!("新しく設定ファイルを作成しました。");
    }
    let config_data =
        fs::read_to_string(config_path).context("設定ファイルが読み込めませんでした。")?;
    let config_data =
        toml::from_str(&config_data).context("設定ファイルがパースできませんでした。")?;
    Ok(config_data)
}

/// 設定を書き込む。
/// 設定ファイルが存在しない場合は作成する。
pub fn save(config: &Config) -> Result<()> {
    let config_path = get_config_path()?;
    fs::create_dir_all(&config_path.parent().unwrap()).context("設定ファイルのあるディレクトリの作成に失敗しました。").unwrap();
    let mut file =
        File::create(config_path).context("設定ファイルの作成または取得に失敗しました。")?;

    let data = toml::to_string(config).context("設定データの文字列化に失敗しました。")?;
    file.write_all(data.as_bytes())
        .context("ファイルへの書き込みに失敗しました。")?;

    Ok(())
}

/// 設定ファイルのパスを取得する。
fn get_config_path() -> Result<PathBuf> {
    Ok(get_project_dirs()?.config_local_dir().join("config.toml"))
}

/// alpacahack-newのプロジェクトディレクトリを取得する。
fn get_project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "alpacahack-new")
        .context("alpacahack-newのプロジェクトディレクトリを取得できませんでした。")
}
