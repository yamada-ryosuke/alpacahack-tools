use reqwest::Url;

/// 問題に関する情報
#[derive(Debug)]
pub struct ChallengeInfo {
    /// 問題に関するメタデータ
    pub meta: ChallengeMeta,
    /// 配布されるデータ
    pub data: Option<ChallengeData>,
}

/// 問題ページで配布されるファイル
#[derive(Debug)]
pub struct ChallengeData {
    /// データのURL
    pub url: Url,
    /// ファイル名
    pub name: String,
    /// データ
    pub data: bytes::Bytes,
}

/// 問題に関する基本情報
#[derive(Debug)]
pub struct ChallengeMeta {
    /// 問題のURL
    pub url: Url,
    /// 問題が出された日付
    pub date: String,
    /// スペースを含む(おそらく正式な)問題の名前
    pub title_with_space: String,
    /// URLやディレクトリ名に使われるkebab-caseの問題の名前
    pub title_with_kebab: String,
}
