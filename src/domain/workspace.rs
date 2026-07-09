use crate::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

struct Workspace {
    path: PathBuf,
}

impl Workspace {
    fn new(path: &Path) -> Result<Self> {
        fs::create_dir_all(path)?;

        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    fn get_daily_dir(&self) -> Result<Daily> {
        let daily_path = self.path.join("daily");
        fs::create_dir_all(&daily_path)?;
        Ok(Daily { path: daily_path })
    }

    fn find_project_by_name() -> Vec<Project> {
        todo!()
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
struct Project {
    path: PathBuf,
}

impl Project {
    /// 初期化ファイルを基にプロジェクトを初期化する。
    fn init(&self) {
        todo!();
    }
}
