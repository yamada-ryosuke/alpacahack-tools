use std::path::PathBuf;

use anyhow::Context;

use crate::{
    cli::{ConfigArgs, ConfigCommands, ConfigSetArgs},
    config::Config,
};

pub fn run(args: ConfigArgs) {
    match args.command {
        Some(ConfigCommands::Set(args)) => set(args),
        Some(ConfigCommands::Init) => init(),
        None => display(),
    }
}

/// 設定を変更する
fn set(args: ConfigSetArgs) {
    let mut config = Config::load()
        .context("現在の設定を取得できませんでした。")
        .unwrap();

    let mut message = String::new();

    // ワークスペースのパスを変更する
    if let Some(path) = args.workspace {
        let path = PathBuf::from(path);
        config.workspace = Some(path.clone());
        message.push_str(&format!(
            "ワークスペースのパスを変更しました。: {}\n",
            path.display()
        ));
    }

    config
        .save()
        .context("設定を保存できませんでした。")
        .unwrap();

    print!("{}", message);
}

/// 設定を初期化する。
fn init() {
    Config::default()
        .save()
        .context("設定の初期化に失敗しました。")
        .unwrap();
}

/// 設定ファイルを表示する。
fn display() {
    let config = Config::load()
        .context("設定を取得できませんでした。")
        .unwrap();
    println!("{}", toml::to_string_pretty(&config).unwrap());
}
