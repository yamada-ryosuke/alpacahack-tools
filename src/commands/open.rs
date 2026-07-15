use crate::{config::Config, editor, prelude::*};

pub fn run(args: crate::cli::OpenArgs) {
    // 設定ファイルからワークスペースのパスを取り出す。
    // ワークスペースの設定が存在しない場合はエラー。
    let config = Config::load().expect("設定を取得できませんでした。");
    let workspace = Workspace::try_from(&config).unwrap();

    if let Some(url) = args.url {
        let url = ChallengeUrl::new(&url).context("不正なURLです。").unwrap();
        let project = workspace
            .search_project_by_url(&url)
            .context("検索に失敗しました。")
            .unwrap();
        match project {
            Some(project) => {
                println!("プロジェクト名: {}", project.name());
                editor::open_with_vscode(&project)
                    .context("VSCodeでディレクトリを開けませんでした。")
                    .unwrap();
            }
            None => println!("プロジェクトが見つかりませんでした。"),
        };
        return;
    }

    panic!("検索のキーを指定してください。");
}
