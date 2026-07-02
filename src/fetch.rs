/// 問題ページから取得したデータを解析する
mod analyse;

use anyhow::Result;
use reqwest::Url;

use crate::prelude::*;

/// URLを基に問題に関するデータを取得する。
///
/// `challenge_url` から問題ページを取得し、ページのメタデータと添付ファイルを
/// 組み合わせて `ChallengeInfo` として返す。
pub fn fetch_challenge_data(challenge_url: &AlpacaHackUrl) -> Result<ChallengeInfo> {
    let (meta, file_url) = fetch_challenge_meta(challenge_url)?;
    let attached = file_url.map(fetch_challenge_attachment).transpose()?;
    Ok(ChallengeInfo { meta, attached })
}

/// 添付ファイルの URL からファイルをダウンロードし、`ChallengeFile` を返す。
fn fetch_challenge_attachment(file_url: Url) -> Result<ChallengeFile> {
    let filename = get_filename(&file_url)?;
    let data = download_file_bytes(&file_url)?;
    Ok(ChallengeFile {
        _url: file_url,
        name: filename,
        data,
    })
}

/// 問題ページから問題のメタデータと添付ファイルの URL を取得する。
///
/// `challenge_url` のページを取得し、`analyse::analyze_document` で解析する。
fn fetch_challenge_meta(challenge_url: &AlpacaHackUrl) -> Result<(ChallengeMeta, Option<Url>)> {
    // challenge_urlからデータを取得する。
    let document = reqwest::blocking::get(challenge_url.as_str())?.text()?;
    // documentから情報を抽出する
    analyse::analyze_document(challenge_url, &document)
}

/// 指定した URL からファイルをダウンロードし、バイト列を返す。
fn download_file_bytes(file_url: &Url) -> Result<bytes::Bytes> {
    Ok(reqwest::blocking::get(file_url.as_str())?.bytes()?)
}

/// ファイルの URL からファイル名を抽出する。
///
/// 最後のパス要素を返す。URL にパスが含まれない場合はエラーになる。
fn get_filename(file_url: &Url) -> Result<String> {
    let filename = file_url
        .path_segments()
        .ok_or(anyhow::anyhow!("URLからパスセグメントを取得できません。"))?
        .next_back()
        .filter(|segment| !segment.is_empty())
        .ok_or(anyhow::anyhow!("URLにファイル名が含まれていません。"))?;
    Ok(filename.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use chrono::NaiveDate;
    use reqwest::Url;

    #[test]
    /// URLの末尾のパス要素からファイル名を正しく取得できることを確認するテスト。
    ///
    /// 例: ".../secret-table.tar.gz" から "secret-table.tar.gz" を返す。
    fn get_filename_returns_last_path_segment() -> Result<()> {
        let url = Url::parse(
            "https://alpacahack-prod.s3.ap-northeast-1.amazonaws.com/69bf0ca5-a858-486a-9fac-f94b65f642a3/secret-table.tar.gz",
        )?;
        assert_eq!(get_filename(&url)?, "secret-table.tar.gz");
        Ok(())
    }

    #[test]
    /// URLにパスが含まれていない場合にエラーを返すことを確認するテスト。
    ///
    /// 例: ホストのみのURLではファイル名が存在しないためエラーとなる。
    fn get_filename_returns_error_when_path_is_empty() -> Result<()> {
        let url = Url::parse("https://alpacahack-prod.s3.ap-northeast-1.amazonaws.com")?;
        let err = get_filename(&url).unwrap_err();
        assert_eq!(err.to_string(), "URLにファイル名が含まれていません。");
        Ok(())
    }

    #[test]
    /// URLがパスセグメントを持たないスキーム（例: mailto）である場合にエラーを返すことを確認するテスト。
    fn get_filename_returns_error_when_path_segments_are_unavailable() -> Result<()> {
        let url = Url::parse("mailto:info@example.com")?;
        let err = get_filename(&url).unwrap_err();
        assert_eq!(err.to_string(), "URLからパスセグメントを取得できません。");
        Ok(())
    }

    #[test]
    /// 指定した Alpacahack のチャレンジURLからメタ情報と添付ファイルURLを正しく抽出できることを確認する統合テスト。
    ///
    /// 実際のページを取得して analyse::analyze_document の結果を検証する。
    fn fetch_challenge_meta_for_rsa2026_url() -> Result<()> {
        let url = AlpacaHackUrl::new("https://alpacahack.com/daily/challenges/rsa2026")?;
        let (meta, attached_url) = fetch_challenge_meta(&url)?;

        let expected_meta = ChallengeMeta {
            url: url,
            released_at: NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
            title: "RSA2026".into(),
            project_name: "rsa2026".into(),
        };
        assert_eq!(meta, expected_meta);

        let expected_attached_url = Url::parse("https://alpacahack-prod.s3.ap-northeast-1.amazonaws.com/76487104-ff52-47f3-84e2-8039513dd6d2/rsa2026.tar.gz").unwrap();
        assert_eq!(attached_url, Some(expected_attached_url));
        Ok(())
    }
}
