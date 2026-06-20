/// 問題ページから取得したデータを解析する
mod analyse;

use anyhow::Result;
use reqwest::Url;

use crate::challenge_info::{ChallengeFile, ChallengeInfo, ChallengeMeta};

/// URLを基に問題に関するデータを取得する。
pub fn fetch_challenge_data(challenge_url: &Url) -> Result<ChallengeInfo> {
    let (metadata, file_url) = fetch_challenge_meta(challenge_url)?;
    match file_url {
        None => Ok(ChallengeInfo {
            meta: metadata,
            attached: None,
        }),
        Some(file_url) => {
            let filename = get_filename(&file_url)?;
            let file_data = download_file(&file_url)?;
            Ok(ChallengeInfo {
                meta: metadata,
                attached: Some(ChallengeFile {
                    _url: file_url,
                    name: filename,
                    data: file_data,
                }),
            })
        }
    }
}

/// 問題ページから問題のメタデータを取得する。
fn fetch_challenge_meta(challenge_url: &Url) -> Result<(ChallengeMeta, Option<Url>)> {
    // challenge_urlからデータを取得する。
    let document = reqwest::blocking::get(challenge_url.as_str())?.text()?;
    // documentから情報を抽出する
    analyse::analyze_document(challenge_url, &document)
}

/// ファイルをダウンロードする。
fn download_file(file_url: &Url) -> Result<bytes::Bytes> {
    Ok(reqwest::blocking::get(file_url.as_str())?.bytes()?)
}

/// ファイルのURLからファイル名を取得する。
fn get_filename(file_url: &Url) -> Result<String> {
    let filename = file_url
        .path_segments()
        .ok_or(anyhow::anyhow!("URLのパスがありません。"))?
        .next_back()
        .ok_or(anyhow::anyhow!("URLのパスが空です。"))?;
    Ok(filename.to_owned())
}
