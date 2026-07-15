use crate::prelude::*;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// 問題ページのURLであることが保証されたURL。
/// クエリパラメータやフラグメントが取り除かれて正規化されている。
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ChallengeUrl(Url);

/// `ChallengeUrl`に対して.as_str()などのUrlのメソッドを直接使えるようにする。
impl Deref for ChallengeUrl {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ChallengeUrl {
    /// URLが正しいことを確かめて、正規化されたURLを返す。
    /// 具体的には以下のことをする。
    /// * URLがURLとしてパースできることを確認する。
    /// * URLが https://alpacahack.com/daily/challenges/<something> の形式であることを確認する。
    /// * クエリパラメータを取り除く。
    /// * フラグメントを取り除く。
    pub fn new(url: &str) -> Result<Self> {
        let mut url = Self::validate_url(url)?;
        Self::normalize_url(&mut url);

        Ok(Self(url))
    }

    /// URLからプロジェクト名に使うkebab-caseの問題タイトルを取得する。
    ///
    /// challenge_url のパス末尾のセグメントをそのまま返す。
    pub fn project_name(&self) -> String {
        let challenge_name = self
            .path_segments()
            .expect("原因不明のバグによりURLのバリデーションが機能していません。")
            .next_back()
            .expect("原因不明のバグによりURLのバリデーションが機能していません。");
        challenge_name.into()
    }

    /// URLを正規化する。
    fn normalize_url(url: &mut Url) {
        // クエリパラメータを取り除く
        url.set_query(None);

        // フラグメントを取り除く
        url.set_fragment(None);

        // 認証情報が付いていたら取り除く
        let _ = url.set_username("");
        let _ = url.set_password(None);

        // デフォルトのポート(HTTPSの443)が明示されていたら取り除く
        let _ = url.set_port(None);

        // ドメインは小文字に正規化する
        if let Some(host) = url.host_str() {
            let _ = url.set_host(Some(&host.to_lowercase()));
        }

        // パスの末尾のスラッシュを取り除く (ただしルート "/" は空にしない)
        let path = url.path().to_string();
        if path.len() > 1 && path.ends_with('/') {
            let new_path = path.trim_end_matches('/');
            url.set_path(new_path);
        }
    }

    /// URLが https://alpacahack.com/daily/challenges/<something> の形式であることを確かめる
    fn validate_url(url: &str) -> Result<Url> {
        let url = Url::parse(url).context("URLのパースに失敗しました。")?;
        Self::validate_scheme(&url)?;
        Self::validate_domain(&url)?;
        Self::validate_path(&url)?;
        Ok(url)
    }

    /// URLのスキームが https であることを確認する。
    fn validate_scheme(url: &Url) -> Result<()> {
        if url.scheme() != "https" {
            return Err(anyhow::anyhow!(
                "URLのスキームは https である必要があります。"
            ));
        }
        Ok(())
    }

    /// URLにドメインがあり、ドメイン名が <https://alpacahack.com> であることを確認する。
    fn validate_domain(url: &Url) -> Result<()> {
        let domain = url
            .domain()
            .ok_or(anyhow::anyhow!("ドメイン名がありません。"))?;
        if domain == "alpacahack.com" {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "ドメイン名が正しくありません。: {}",
                domain
            ))
        }
    }

    /// URLのパスが /daily/challenges/<something> の形式であることを確認する。
    fn validate_path(url: &Url) -> Result<()> {
        let segments: Vec<_> = url
            .path_segments()
            .ok_or(anyhow::anyhow!("URLのパスが取得できませんでした。"))?
            .collect();

        if segments.len() != 3
            || segments[0] != "daily"
            || segments[1] != "challenges"
            || segments[2].is_empty()
        {
            return Err(anyhow::anyhow!(
                "URLの形式は https://alpacahack.com/daily/challenges/<something> の必要があります。"
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== project_name() のテスト ====================
    /// URLの末尾からプロジェクト名（kebab-case）を正しく抽出できることを確認するテスト
    #[test]
    fn test_project_name_valid_url() {
        let url =
            ChallengeUrl::new("https://alpacahack.com/daily/challenges/secret-table").unwrap();
        let project_name = url.project_name();
        assert_eq!(project_name, "secret-table");
    }

    #[test]
    fn test_project_name_with_numbers() {
        let url = ChallengeUrl::new("https://alpacahack.com/daily/challenges/small-e").unwrap();
        assert_eq!(url.project_name(), "small-e");
    }

    #[test]
    fn test_project_name_single_word() {
        let url = ChallengeUrl::new(
            "https://alpacahack.com/daily/challenges/message-for-you?month=2026-04",
        )
        .unwrap();
        assert_eq!(url.project_name(), "message-for-you");
    }

    // ==================== new() の正常系テスト ====================
    #[test]
    fn test_new_valid_url() {
        let result = ChallengeUrl::new("https://alpacahack.com/daily/challenges/vm1");
        assert!(result.is_ok());
    }

    /// クエリパラメータとフラグメントが削除されることを確認するテスト
    #[test]
    fn test_new_normalizes_query_and_fragment() {
        let url = ChallengeUrl::new(
            "https://alpacahack.com/daily/challenges/alpaca-rangers-2?month=2026-05#section",
        )
        .unwrap();

        // URLのクエリとフラグメントが削除されていることを確認
        assert_eq!(url.query(), None);
        assert_eq!(url.fragment(), None);
        assert_eq!(
            url.as_str(),
            "https://alpacahack.com/daily/challenges/alpaca-rangers-2"
        );
    }

    /// クエリパラメータだけが付いているURLを正規化できることを確認するテスト
    #[test]
    fn test_new_normalizes_query_only() {
        let url =
            ChallengeUrl::new("https://alpacahack.com/daily/challenges/small-n?month=2026-05&b=2")
                .unwrap();
        assert_eq!(url.query(), None);
    }

    /// フラグメントだけが付いているURLを正規化できることを確認するテスト
    #[test]
    fn test_new_normalizes_fragment_only() {
        let url =
            ChallengeUrl::new("https://alpacahack.com/daily/challenges/func-array#top").unwrap();
        assert_eq!(url.fragment(), None);
    }

    // ==================== scheme のバリデーションテスト ====================
    #[test]
    fn test_new_rejects_http_scheme() {
        let result = ChallengeUrl::new("http://alpacahack.com/daily/challenges/alpaca-plus-plus");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("https である必要があります")
        );
    }

    #[test]
    fn test_new_rejects_ftp_scheme() {
        let result = ChallengeUrl::new("ftp://alpacahack.com/daily/challenges/alpaca-plus-plus");
        assert!(result.is_err());
    }

    // ==================== domain のバリデーションテスト ====================
    #[test]
    fn test_new_rejects_wrong_domain() {
        let result = ChallengeUrl::new("https://example.com/daily/challenges/alpaca-plus-plus");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("ドメイン名が正しくありません")
        );
    }

    #[test]
    fn test_new_rejects_subdomain() {
        let result =
            ChallengeUrl::new("https://api.alpacahack.com/daily/challenges/alpaca-plus-plus");
        assert!(result.is_err());
    }

    // ==================== path のバリデーションテスト ====================
    #[test]
    fn test_new_rejects_wrong_path_structure() {
        let result = ChallengeUrl::new("https://alpacahack.com/challenges/test");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("/daily/challenges/<something>")
        );
    }

    #[test]
    fn test_new_rejects_missing_challenge_name() {
        let result = ChallengeUrl::new("https://alpacahack.com/daily/challenges/");
        assert!(result.is_err());
    }

    #[test]
    fn test_new_rejects_path_with_trailing_slash() {
        let result = ChallengeUrl::new("https://alpacahack.com/daily/challenges/alpaca-plus-plus/");
        assert!(result.is_err());
    }

    #[test]
    fn test_new_rejects_extra_path_segments() {
        let result =
            ChallengeUrl::new("https://alpacahack.com/daily/challenges/alpaca-plus-plus/extra");
        assert!(result.is_err());
    }

    // ==================== Deref トレイトのテスト ====================
    #[test]
    fn test_deref_url_methods() {
        let url = ChallengeUrl::new("https://alpacahack.com/daily/challenges/test").unwrap();

        // Derefを通じてUrl のメソッドが使えることを確認
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.domain(), Some("alpacahack.com"));
        assert_eq!(url.host_str(), Some("alpacahack.com"));
    }

    #[test]
    fn test_deref_as_str() {
        let url = ChallengeUrl::new("https://alpacahack.com/daily/challenges/test").unwrap();
        assert_eq!(url.as_str(), "https://alpacahack.com/daily/challenges/test");
    }

    // ==================== EquaityとCloneのテスト ====================
    #[test]
    fn test_equality() {
        let url1 = ChallengeUrl::new("https://alpacahack.com/daily/challenges/test").unwrap();
        let url2 = ChallengeUrl::new("https://alpacahack.com/daily/challenges/test").unwrap();
        assert_eq!(url1, url2);
    }

    #[test]
    fn test_inequality() {
        let url1 = ChallengeUrl::new("https://alpacahack.com/daily/challenges/test1").unwrap();
        let url2 = ChallengeUrl::new("https://alpacahack.com/daily/challenges/test2").unwrap();
        assert_ne!(url1, url2);
    }

    #[test]
    fn test_clone() {
        let url1 = ChallengeUrl::new("https://alpacahack.com/daily/challenges/test").unwrap();
        let url2 = url1.clone();
        assert_eq!(url1, url2);
        assert_eq!(url1.project_name(), url2.project_name());
    }

    // ==================== 無効なURL形式のテスト ====================
    #[test]
    fn test_new_rejects_invalid_url_format() {
        let result = ChallengeUrl::new("not a url");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("URLのパースに失敗しました")
        );
    }

    #[test]
    fn test_new_rejects_empty_string() {
        let result = ChallengeUrl::new("");
        assert!(result.is_err());
    }
}
