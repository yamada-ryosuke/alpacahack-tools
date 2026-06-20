use anyhow::Result;
use reqwest::Url;

use crate::problem_info::{ChallengeData, ChallengeInfo};

/// URLからデータを取得する。
pub async fn fetch_problem_data(file_url: &Url) -> Result<ChallengeInfo> {
    let filename = get_filename(file_url)?;
    let file_data = download(file_url).await?;
    Ok(ChallengeInfo {
        problem_name_with_kebab: filename.clone(),
        data: ChallengeData {
            name: filename,
            data: file_data,
        },
    })
}

/// ファイルをダウンロードする。
async fn download(url: &Url) -> Result<bytes::Bytes> {
    Ok(reqwest::get(url.as_str()).await?.bytes().await?)
}

/// URLからファイル名を取得する。
fn get_filename(url: &Url) -> Result<String> {
    let filename = url
        .path_segments()
        .ok_or(anyhow::anyhow!("URLのパスがありません。"))?
        .next_back()
        .ok_or(anyhow::anyhow!("URLのパスが空です。"))?;
    Ok(filename.to_owned())
}
