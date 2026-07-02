use std::path::PathBuf;

use anyhow::Context;

use crate::cli::ConfigArgs;

pub fn run(args: ConfigArgs) {
    let mut config = crate::config::load()
        .context("設定を取得できませんでした。")
        .unwrap();

    // ワークスペースのパスを変更する
    if let Some(path) = args.workspace {
        config.workspace = PathBuf::from(path);
        println!("ワークスペースのパスを変更します。: {}", config.workspace.display());
    }

    crate::config::save(&config)
        .context("設定を保存できませんでした。")
        .unwrap();
}
