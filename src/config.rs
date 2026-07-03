use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

/// alpacahack-toolsの設定
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    /// 各問題のプロジェクトを展開するワークスペースディレクトリのパス
    pub workspace: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self { workspace: None }
    }
}

/// 設定を取得する。
/// 設定ファイルが存在しない場合は設定ファイルを取得する。
pub fn load() -> Result<Config> {
    let config_path = get_config_path()?;
    // 設定ファイルがなければ作成する。
    if !exists() {
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
    let mut file = create_config_file()?;

    let data = toml::to_string(config).context("設定データの文字列化に失敗しました。")?;
    file.write_all(data.as_bytes())
        .context("ファイルへの書き込みに失敗しました。")?;

    Ok(())
}

/// 設定ファイルが存在するか。
/// モジュール外からは「設定ファイルが存在していない」と「特定の項目が設定されていない」の区別はつかなくていいので、非公開になってる。
fn exists() -> bool {
    get_config_path().unwrap().exists()
}

/// 設定ファイルを作成する。
/// 存在する場合はファイルを書き込みモードで取得する。
fn create_config_file() -> Result<File> {
    let config_path = get_config_path()?;
    fs::create_dir_all(config_path.parent().unwrap())
        .context("設定ファイルを置くディレクトリの作成に失敗しました。")
        .unwrap();
    File::create(config_path).context("設定ファイルの作成または取得に失敗しました。")
}

/// 設定ファイルのパスを取得する。
fn get_config_path() -> Result<PathBuf> {
    Ok(get_project_dirs()?.config_local_dir().join("config.toml"))
}

/// alpacahack-toolsのプロジェクトディレクトリを取得する。
fn get_project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "alpacahack-tools")
        .context("alpacahack-toolsのプロジェクトディレクトリを取得できませんでした。")
}
