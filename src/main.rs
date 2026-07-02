use clap::Parser;

/// AlpacaHackのURLの構造体のあるモジュール
mod alpacahack_url;
/// 問題の情報を持つための構造体
mod challenge_info;
/// 設定ファイルを管理するモジュール
mod config;
/// ユビキタス言語っぽいやつ
mod prelude;

/// 各サブコマンドをまとめたモジュール
mod commands;

/// コマンドライン引数
mod cli;

fn main() {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::New(args) => commands::new::run(args),
        cli::Commands::Config(args) => commands::config::run(args),
    };
}
