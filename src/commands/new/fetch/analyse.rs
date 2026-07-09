use chrono::NaiveDate;
use reqwest::Url;
use scraper::Html;

use crate::prelude::*;

/// 問題ページを解析して、必要なデータを抽出する。
///
/// # 引数
/// - challenge_url: 元の問題ページのURL
/// - document: HTML文字列
///
/// # 返り値
/// 返り値は ChallengeMeta とオプションのファイルURL。
pub fn analyze_document(
    challenge_url: &AlpacaHackUrl,
    document: &str,
) -> Result<(ChallengeMeta, Option<Url>)> {
    let document = scraper::Html::parse_document(document);
    // スクレイピングするためにセレクタを作成する際、おそらく動的生成のためセレクタが崩れやすい。
    // そのため、`main > div > h1`程度のアバウトなセレクタを使う。
    Ok((
        ChallengeMeta {
            url: challenge_url.clone(),
            released_at: get_date(&document)?,
            title: get_title(&document)?,
            project_name: challenge_url.project_name(),
        },
        get_file_url(&document)?,
    ))
}

/// 問題の正式名称を取得する。
///
/// HTML の `main > div > h1` 要素からタイトル文字列を取り出す。
fn get_title(document: &Html) -> Result<String> {
    let title_selector = scraper::Selector::parse("main > div > h1")
        .map_err(|_| anyhow::anyhow!("問題タイトルのセレクタの作成に失敗しました"))?;

    let parent = document
        .select(&title_selector)
        .next()
        .ok_or(anyhow::anyhow!(
            "問題タイトルのh1要素を取得できませんでした。"
        ))?;

    Ok(parent.inner_html())
}

/// ファイルのURLを取得する
///
/// HTML の `main > div > article > div > div > a` から href 属性を解析する。
fn get_file_url(document: &Html) -> Result<Option<Url>> {
    let file_url_selector = scraper::Selector::parse("main > div > article > div > div > a")
        .map_err(|_| anyhow::anyhow!("ファイルのURLのセレクタの作成に失敗しました"))?;

    let parent = document.select(&file_url_selector).next();

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
///
/// HTML の `main > div > p` 要素内の最終子要素を日付文字列として扱う。
fn get_date(document: &Html) -> Result<NaiveDate> {
    let release_selector = scraper::Selector::parse("main > div > p").unwrap();

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

/// 文字列を指定されたフォーマットでNaiveDateに変換する。
/// 入力フォーマット: "%b %e, %Y" (例: "Jan  1, 2024")
fn convert_to_naive_date(date_string: &str) -> Result<NaiveDate> {
    let date = NaiveDate::parse_from_str(date_string, "%b %e, %Y")?;
    Ok(date)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 正しい形式の日付文字列をNaiveDateに変換できることを確認するテスト
    #[test]
    fn test_convert_to_naive_date_valid() {
        let result = convert_to_naive_date("Jan  1, 2024");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
        );
    }

    /// 無効な形式の日付文字列はエラーになることを確認するテスト
    #[test]
    fn test_convert_to_naive_date_invalid_format() {
        let result = convert_to_naive_date("2024-01-01");
        assert!(result.is_err());
    }
}
