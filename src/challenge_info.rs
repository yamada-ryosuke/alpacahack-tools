use chrono::NaiveDate;
use reqwest::Url;
use serde::Serialize;

/// 問題に関する情報
#[derive(Debug)]
pub struct ChallengeInfo {
    /// 問題に関するメタデータ
    pub meta: ChallengeMeta,
    /// 配布されるファイル
    pub attached: Option<ChallengeFile>,
}

/// 問題に関する基本情報
#[derive(Debug, Serialize)]
pub struct ChallengeMeta {
    /// 問題のURL
    pub url: Url,
    /// 問題が出された日付
    pub released_at: NaiveDate,
    /// スペースを含む(おそらく正式な)問題の名前
    pub title: String,
    /// URLやディレクトリ名に使われるkebab-caseの問題の名前
    pub project_name: String,
}

/// 問題ページで配布されるファイル
#[derive(Debug)]
pub struct ChallengeFile {
    /// データのURL
    pub _url: Url,
    /// ファイル名
    pub name: String,
    /// データ
    pub data: bytes::Bytes,
}
