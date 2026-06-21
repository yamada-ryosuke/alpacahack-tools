use anyhow::{Context, Result};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// AlpacaHackのURLであることが保証されたURL。
/// クエリパラメータやフラグメントが取り除かれて正規化されている。
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AlpacaHackUrl {
    /// URL
    url: Url,
}

/// `AlpacaHackUrl`に対して.as_str()などのUrlのメソッドを直接使えるようにする。
impl Deref for AlpacaHackUrl {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.url
    }
}

impl AlpacaHackUrl {
    /// URLが正しいことを確かめて、正規化されたURLを返す。
    /// 具体的には以下のことをする。
    /// * URLがURLとしてパースできることを確認する。
    /// * URLが https://alpacahack.com/daily/challenges/<something> の形式であることを確認する。
    /// * クエリパラメータを取り除く。
    /// * フラグメントを取り除く。
    pub fn new(url: &str) -> Result<Self> {
        let mut url = Self::validate_url(url)?;
        Self::normalize_url(&mut url);

        Ok(Self { url })
    }

    /// URLからプロジェクト名に使うkebab-caseの問題タイトルを取得する。
    ///
    /// challenge_url のパス末尾のセグメントをそのまま返す。
    pub fn project_name(&self) -> String {
        let challenge_name = self
            .path_segments()
            .expect("原因不明のバグによりURLのバリデーションが機能していません。")
            .filter(|segment| !segment.is_empty())
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
    /// URLの末尾からプロジェクト名（kebab-case）を正しく抽出できることを確認するテスト
    #[test]
    fn test_project_name_valid_url() {
        let url = AlpacaHackUrl::new("https://alpacahack.com/daily/challenges/secret-table").unwrap();
        let project_name = url.project_name();
        assert_eq!(project_name, "secret-table");
    }

    
}