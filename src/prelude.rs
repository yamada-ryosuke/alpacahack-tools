/// AlpacaHackのURLの構造体のあるモジュール
pub(crate) mod alpacahack_url;
/// 問題の情報を持つための構造体
pub(crate) mod challenge_info;

pub(crate) use alpacahack_url::AlpacaHackUrl;
pub(crate) use challenge_info::{ChallengeFile, ChallengeInfo, ChallengeMeta};
