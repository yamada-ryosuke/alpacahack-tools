use clap::{Args, Parser, Subcommand};

/// AlpacaHackのプロジェクトディレクトリを作成するプログラム。
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// サブコマンド
#[derive(Subcommand)]
pub enum Commands {
    /// プロジェクトディレクトリを作成する。
    New(NewArgs),
    /// 設定を変更する。
    Config(ConfigArgs),
}

/// サブコマンド`new`のコマンドライン引数
#[derive(Args)]
pub struct NewArgs {
    /// 問題URLを指定する。指定されない場合は対話的に入力することになる。
    #[arg(long, short)]
    pub url: Option<String>,
}

/// サブコマンド`config`のコマンドライン引数
#[derive(Args)]
pub struct ConfigArgs {
    /// 各プロジェクトを展開するワークスペースディレクトリの絶対パス。
    #[arg(long)]
    pub workspace: Option<String>
}
