use std::path::PathBuf;

use crate::prelude::*;

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

/// 設定変更コマンドを実行し、必要なら変更内容を表示する。
fn set(args: ConfigSetArgs) {
    let mut config = Config::load()
        .context("現在の設定を取得できませんでした。")
        .unwrap();

    let message = apply_config_changes(&mut config, args);

    config
        .save()
        .context("設定を保存できませんでした。")
        .unwrap();

    print!("{}", message);
}

/// 1回の設定変更処理で発生した変更内容を表す。
#[derive(Debug, PartialEq, Eq)]
enum ConfigChange {
    /// ワークスペースのパスが更新されたことを表す。
    WorkspaceUpdated(PathBuf),
}

impl ConfigChange {
    /// この変更内容をユーザー向けのメッセージに変換する。
    fn message(&self) -> String {
        match self {
            Self::WorkspaceUpdated(path) => {
                format!("ワークスペースのパスを変更しました。: {}\n", path.display())
            }
        }
    }
}

/// 引数に基づいて設定を更新し、発生した変更をまとめてメッセージ化する。
fn apply_config_changes(config: &mut Config, args: ConfigSetArgs) -> String {
    let mut changes = Vec::new();

    // ワークスペースのパスが指定されていれば、設定を更新して変更履歴に追加する。
    if let Some(path) = args.workspace {
        let path = PathBuf::from(path);
        config.workspace = Some(path.clone());
        changes.push(ConfigChange::WorkspaceUpdated(path));
    }

    format_changes(&changes)
}

/// 蓄えた変更内容を、出力用のメッセージ文字列に整形する。
fn format_changes(changes: &[ConfigChange]) -> String {
    changes.iter().map(ConfigChange::message).collect()
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{apply_config_changes, format_changes, ConfigChange};
    use crate::{cli::ConfigSetArgs, config::Config};

    #[test]
    /// ワークスペースが指定された場合に、設定が更新されることを確認する。
    fn updates_workspace_when_workspace_is_provided() {
        let mut config = Config::default();
        let args = ConfigSetArgs {
            workspace: Some("/tmp/workspace".to_string()),
        };

        let message = apply_config_changes(&mut config, args);

        assert_eq!(config.workspace, Some(PathBuf::from("/tmp/workspace")));
        assert_eq!(
            message,
            "ワークスペースのパスを変更しました。: /tmp/workspace\n"
        );
    }

    #[test]
    /// 蓄えた変更内容から、期待するメッセージが生成されることを確認する。
    fn renders_workspace_update_message_from_accumulated_changes() {
        let changes = [ConfigChange::WorkspaceUpdated(PathBuf::from("/tmp/workspace"))];

        let message = format_changes(&changes);

        assert_eq!(
            message,
            "ワークスペースのパスを変更しました。: /tmp/workspace\n"
        );
    }

    #[test]
    /// ワークスペースが指定されない場合に、設定が変化せず空メッセージになることを確認する。
    fn keeps_workspace_unchanged_when_workspace_is_not_provided() {
        let mut config = Config::default();
        let args = ConfigSetArgs { workspace: None };

        let message = apply_config_changes(&mut config, args);

        assert_eq!(config.workspace, None);
        assert!(message.is_empty());
    }
}
