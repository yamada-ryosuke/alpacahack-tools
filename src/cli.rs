use clap::{Args, Parser, Subcommand};

/// AlpacaHackのプロジェクトディレクトリを作成するプログラム。
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// `alpacahack-tools`のサブコマンド
#[derive(Subcommand)]
pub enum Commands {
    /// プロジェクトディレクトリを作成する。
    New(NewArgs),
    /// 設定を変更する。
    Config(ConfigArgs),
}

/// `alpacahack-tools new`のコマンドライン引数
#[derive(Args)]
pub struct NewArgs {
    /// 問題URLを指定する。指定されない場合は対話的に入力することになる。
    #[arg(long, short)]
    pub url: Option<String>,
}

/// `alpacahack-tools config`のコマンドライン引数
#[derive(Args)]
pub struct ConfigArgs {
    /// `alpacahack-tools config`のサブコマンド
    /// サブコマンドが指定されない場合は設定ファイルを表示する。
    #[command(subcommand)]
    pub command: Option<ConfigCommands>
}

/// alpacahack-tools configのサブコマンド
#[derive(Subcommand)]
pub enum ConfigCommands {
    Set(ConfigSetArgs),
    Init,
}

#[derive(Args)]
pub struct ConfigSetArgs {
    /// 各プロジェクトを展開するワークスペースディレクトリの絶対パス。
    #[arg(long)]
    pub workspace: Option<String>,
}
