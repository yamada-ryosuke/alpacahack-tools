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
    /// プロジェクトディレクトリを開く。
    // 命名の候補として他にsearchコマンドもありえるが、「開く」という副作用があることを考えるとopenの方がいいかなと思った。
    Open(OpenArgs),
    /// 設定を変更、閲覧する。
    Config(ConfigArgs),
}

/// `alpacahack-tools new`のコマンドライン引数
#[derive(Args)]
pub struct NewArgs {
    /// 問題URLを指定する。指定されない場合は対話的に入力することになる。
    #[arg(long, short)]
    pub url: Option<String>,
}

/// `alpacahack-tools open`のコマンドライン引数
#[derive(Args)]
pub struct OpenArgs {
    /// 問題URLでプロジェクトを検索する。
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
    /// 設定を変更する。
    Set(ConfigSetArgs),
    /// 設定を初期化する。
    Init,
}

#[derive(Args)]
pub struct ConfigSetArgs {
    /// 各プロジェクトを展開するワークスペースディレクトリを絶対パスで指定する。
    #[arg(long)]
    pub workspace: Option<String>,
}
