use clap::Parser;

/// 設定ファイルを管理するモジュール
mod config;
/// ユビキタス言語っぽいやつ
mod domain;
/// 大体のモジュールで使うやつ
mod prelude;
/// 各サブコマンドをまとめたモジュール
mod commands;
/// コマンドライン引数
mod cli;
/// エディター
mod editor;

fn main() {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::New(args) => commands::new::run(args),
        cli::Commands::Config(args) => commands::config::run(args),
        cli::Commands::Open(args) => commands::open::run(args),
    };
}
