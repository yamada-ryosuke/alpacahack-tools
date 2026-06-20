use anyhow::Result;
use chrono::NaiveDate;
use reqwest::Url;
use scraper::Html;

use crate::challenge_info::ChallengeMeta;

/// 問題ページを解析して、必要なデータを抽出する。
pub fn analyze_document(
    challenge_url: &Url,
    document: &str,
) -> Result<(ChallengeMeta, Option<Url>)> {
    let document = scraper::Html::parse_document(document);
    // スクレイピングするためにセレクタを作成する際、おそらく動的生成のためセレクタが崩れやすい。
    // そのため、`main > div > h1`程度のアバウトなセレクタを使う。
    Ok((
        ChallengeMeta {
            url: challenge_url.clone(),
            released_at: get_date(&document)?,
            title: get_name_with_space(&document)?,
            project_name: get_name_with_kebab(challenge_url)?,
        },
        get_file_url(&document)?,
    ))
}

/// 問題の正式名称を取得する。
fn get_name_with_space(document: &Html) -> Result<String> {
    // 親要素のセレクタを作成する。
    let name_with_space_selector = scraper::Selector::parse("main > div > h1")
        .map_err(|_| anyhow::anyhow!("問題タイトルのセレクタの作成に失敗しました"))?;

    // 親要素を取得する
    let parent = document
        .select(&name_with_space_selector)
        .next()
        .ok_or(anyhow::anyhow!(
            "問題タイトルのh1要素を取得できませんでした。"
        ))?;
    // 名前を取得する
    Ok(parent.inner_html())
}

/// ファイルのURLを取得する
fn get_file_url(document: &Html) -> Result<Option<Url>> {
    // 親要素のセレクタを作成する
    let file_url_selector = scraper::Selector::parse("main > div > article > div > div > a")
        .map_err(|_| anyhow::anyhow!("ファイルのURLのセレクタの作成に失敗しました"))?;

    // 親要素を取得する
    let parent = document.select(&file_url_selector).next();
    // 親要素があればURLを取得して返す。なければURLなしと判断してNoneを返す。
    match parent {
        None => Ok(None),
        Some(parent) => {
            let href = parent
                .attr("href")
                .ok_or(anyhow::anyhow!("ファイルのURLが取得できません"))?;
            Ok(Some(Url::parse(href)?))
        }
    }
}

/// 問題の日付を取得する
fn get_date(document: &Html) -> Result<NaiveDate> {
    // 親要素のセレクタを作成する。
    let release_selector = scraper::Selector::parse("main > div > p").unwrap();

    // 親要素を取得する。
    let parent = document
        .select(&release_selector)
        .next()
        .ok_or(anyhow::anyhow!("日付の親のp要素を取得できませんでした。"))?;
    let date_elem = parent
        .last_child()
        .ok_or(anyhow::anyhow!("日付の要素を取得できませんでした。"))?
        .value();
    let date_string = date_elem
        .as_text()
        .ok_or(anyhow::anyhow!(
            "日付の要素をテキストに変換できませんでした。"
        ))?
        .to_string();
    convert_to_naive_date(&date_string)
}

fn convert_to_naive_date(date_string: &str) -> Result<NaiveDate> {
    let date = NaiveDate::parse_from_str(date_string, "%b %e, %Y")?;
    Ok(date)
}

/// URLからkebab-caseの問題タイトルを取得する。
fn get_name_with_kebab(challenge_url: &Url) -> Result<String> {
    let challenge_name = challenge_url
        .path_segments()
        .ok_or(anyhow::anyhow!("URLのパスがありません。"))?
        .next_back()
        .ok_or(anyhow::anyhow!("URLのパスが空です。"))?;
    Ok(challenge_name.to_owned())
}
