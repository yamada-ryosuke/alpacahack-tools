use std::path::PathBuf;

use anyhow::Context;

use crate::cli::ConfigArgs;

pub fn run(args: ConfigArgs) {
    let mut config = crate::config::load()
        .context("設定を取得できませんでした。")
        .unwrap();

    let mut message = String::new();

    // ワークスペースのパスを変更する
    if let Some(path) = args.workspace {
        let path = PathBuf::from(path);
        config.workspace = Some(path.clone());
        message.push_str(&format!("ワークスペースのパスを変更しました。: {}\n", path.display()));
    }

    crate::config::save(&config)
        .context("設定を保存できませんでした。")
        .unwrap();

    println!("{}", toml::to_string_pretty(&config).unwrap());
}
