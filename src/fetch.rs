use anyhow::Result;
use reqwest::Url;

use crate::challenge_info::{ChallengeData, ChallengeInfo, ChallengeMeta};

/// URLを基に問題に関するデータを取得する。
pub fn fetch_challenge_data(challenge_url: &Url) -> Result<ChallengeInfo> {
    let (metadata, file_url) = fetch_challenge_meta(challenge_url)?;
    match file_url {
        None => Ok(ChallengeInfo {
            meta: metadata,
            data: None,
        }),
        Some(file_url) => {
            let filename = get_filename(&file_url)?;
            let file_data = download_file(&file_url)?;
            Ok(ChallengeInfo {
                meta: metadata,
                data: Some(ChallengeData {
                    url: file_url,
                    name: filename,
                    data: file_data,
                }),
            })
        }
    }
}

/// 問題ページから問題のメタデータを取得する。
fn fetch_challenge_meta(challenge_url: &Url) -> Result<(ChallengeMeta, Option<Url>)> {
    // challenge_urlからデータを取得してパースする
    let document = reqwest::blocking::get(challenge_url.as_str())?.text()?;
    let document = scraper::Html::parse_document(&document);

    // セレクタを作成する
    // おそらく動的生成のため、セレクタが崩れやすい。
    // そのため、`main > div > h1`程度のアバウトなセレクタを使う。
    let name_with_space_selector = scraper::Selector::parse("main > div > h1").unwrap();
    let file_url_selector =
        scraper::Selector::parse("main > div > article > div > div > a").unwrap();
    // 日付の上のp要素のセレクタ
    let release_selector = scraper::Selector::parse("main > div > p").unwrap();

    // データを取得する
    let name_with_space = document
        .select(&name_with_space_selector)
        .next()
        .unwrap()
        .inner_html();
    let file_url = document
        .select(&file_url_selector)
        .next()
        .and_then(|elem| elem.attr("href"))
        .map(|s| Url::parse(s).unwrap());

    let date = document
        .select(&release_selector)
        .next()
        .unwrap()
        .last_child()
        .unwrap()
        .value()
        .as_text()
        .unwrap()
        .to_string();

    let name_with_kebab = get_name_with_kebab(challenge_url)?;
    Ok((
        ChallengeMeta {
            url: challenge_url.clone(),
            date,
            name_with_space,
            name_with_kebab,
        },
        file_url,
    ))
}

/// ファイルをダウンロードする。
fn download_file(file_url: &Url) -> Result<bytes::Bytes> {
    Ok(reqwest::blocking::get(file_url.as_str())?.bytes()?)
}

/// URLからファイル名を取得する。
fn get_filename(file_url: &Url) -> Result<String> {
    let filename = file_url
        .path_segments()
        .ok_or(anyhow::anyhow!("URLのパスがありません。"))?
        .next_back()
        .ok_or(anyhow::anyhow!("URLのパスが空です。"))?;
    Ok(filename.to_owned())
}

/// URLからkebab-caseの問題名を取得する。
fn get_name_with_kebab(challenge_url: &Url) -> Result<String> {
    let challenge_name = challenge_url
        .path_segments()
        .ok_or(anyhow::anyhow!("URLのパスがありません。"))?
        .next_back()
        .ok_or(anyhow::anyhow!("URLのパスが空です。"))?;
    Ok(challenge_name.to_owned())
}
