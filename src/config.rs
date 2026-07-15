#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;


use serde::{Deserialize, Serialize};

use crate::prelude::*;

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

    /// 指定したパスから設定を取得する。
    /// 設定ファイルが存在しない場合は設定ファイルを作成してから取得する。
    #[cfg(test)]
    pub fn load_from_path(path: &Path) -> Result<Self> {
        file::load_from_path(path)
    }

    /// 設定を書き込む。
    /// 設定ファイルが存在しない場合は作成する。
    pub fn save(&self) -> Result<()> {
        file::save(self)
    }

    /// 指定したパスに設定を書き込む。
    /// 設定ファイルが存在しない場合は作成する。
    #[cfg(test)]
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        file::save_to_path(self, path)
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
        fs,
        path::{Path, PathBuf},
    };

    use super::Config;
    use directories::ProjectDirs;
    use crate::prelude::*;

    /// 設定を取得する。
    /// 設定ファイルが存在しない場合は設定ファイルを取得する。
    pub(super) fn load() -> Result<Config> {
        load_from_path(&get_config_file_path()?)
    }

    pub(super) fn load_from_path(config_path: &Path) -> Result<Config> {
        // 設定ファイルがなければ作成する。
        if !config_path.exists() {
            save_to_path(&Config::default(), config_path)?;
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
        save_to_path(config, &get_config_file_path()?)
    }

    pub(super) fn save_to_path(config: &Config, config_path: &Path) -> Result<()> {
        // ディレクトリを作成する。
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("設定ファイルを置くディレクトリの作成に失敗しました。")?;
        }

        let data = toml::to_string(config).context("設定データの文字列化に失敗しました。")?;
        fs::write(config_path, data.as_bytes())
            .context("ファイルへの書き込みに失敗しました。")?;

        Ok(())
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

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use tempfile::tempdir;

    use super::Config;

    #[test]
    fn saves_and_loads_config_from_an_explicit_path() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let expected = Config {
            workspace: Some(PathBuf::from("/tmp/workspace")),
        };

        expected.save_to_path(&config_path).unwrap();
        let loaded = Config::load_from_path(&config_path).unwrap();

        assert_eq!(loaded, expected);
        assert!(config_path.exists());
    }

    #[test]
    fn creates_a_default_config_file_when_the_target_does_not_exist() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let loaded = Config::load_from_path(&config_path).unwrap();

        assert_eq!(loaded, Config::default());
        assert!(config_path.exists());

        let written = fs::read_to_string(&config_path).unwrap();
        let parsed: Config = toml::from_str(&written).unwrap();
        assert_eq!(parsed, Config::default());
    }
}
