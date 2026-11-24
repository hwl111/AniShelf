use std::sync::OnceLock;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

const BASE_URL: &str = "https://api.bgm.tv";
const USER_AGENT: &str = "AniShelf/0.1.0 (local personal desktop app)";
const MIN_REQUEST_INTERVAL: Duration = Duration::from_secs(1);

static THROTTLE: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();

#[derive(Debug, thiserror::Error)]
pub enum BangumiError {
    #[error("无法连接 Bangumi，请检查网络")]
    Network,
    #[error("Bangumi 请求失败（HTTP {0}）")]
    Http(u16),
    #[error("未找到该作品")]
    NotFound,
    #[error("Bangumi 请求过于频繁，请稍后重试")]
    RateLimited,
    #[error("Bangumi 返回数据无法解析")]
    Parse,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BangumiSubject {
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub name_cn: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub eps: Option<i64>,
    #[serde(default)]
    pub images: Option<BangumiImages>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BangumiImages {
    #[serde(default)]
    pub large: String,
    #[serde(default)]
    pub common: String,
    #[serde(default)]
    pub medium: String,
}

impl BangumiSubject {
    pub fn display_title(&self) -> &str {
        let name_cn = self.name_cn.trim();
        if name_cn.is_empty() {
            self.name.trim()
        } else {
            name_cn
        }
    }

    pub fn original_title(&self) -> Option<String> {
        let name = self.name.trim();
        if name.is_empty() || name == self.display_title() {
            None
        } else {
            Some(name.to_string())
        }
    }

    pub fn cover_url(&self) -> Option<String> {
        let images = self.images.as_ref()?;
        [&images.large, &images.common, &images.medium]
            .into_iter()
            .map(|url| url.trim())
            .find(|url| !url.is_empty())
            .map(str::to_string)
    }

    pub fn year_and_season(&self) -> (Option<i64>, Option<String>) {
        let Some(date) = self.date.as_deref().map(str::trim) else {
            return (None, None);
        };
        if date.len() < 7 {
            return (None, None);
        }

        let year = date.get(0..4).and_then(|value| value.parse::<i64>().ok());
        let season = date
            .get(5..7)
            .and_then(|value| value.parse::<u32>().ok())
            .and_then(season_for_month)
            .map(str::to_string);

        (year, season)
    }

    pub fn total_episodes(&self) -> Option<i64> {
        self.eps.filter(|episodes| *episodes > 0)
    }
}

fn season_for_month(month: u32) -> Option<&'static str> {
    match month {
        1..=3 => Some("winter"),
        4..=6 => Some("spring"),
        7..=9 => Some("summer"),
        10..=12 => Some("fall"),
        _ => None,
    }
}

#[derive(Serialize)]
struct SearchRequest<'a> {
    keyword: &'a str,
    sort: &'static str,
    filter: SearchFilter,
}

#[derive(Serialize)]
struct SearchFilter {
    #[serde(rename = "type")]
    subject_type: Vec<u8>,
}

#[derive(Deserialize)]
struct SearchResponse {
    #[serde(default)]
    data: Vec<BangumiSubject>,
}

pub struct BangumiClient {
    http: reqwest::Client,
    token: Option<String>,
}

impl BangumiClient {
    pub fn new(token: Option<String>) -> Result<Self, BangumiError> {
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|error| {
                eprintln!("[AniShelf] 初始化 Bangumi 客户端失败：{error}");
                BangumiError::Network
            })?;

        Ok(Self { http, token })
    }

    pub async fn search(&self, keyword: &str) -> Result<Vec<BangumiSubject>, BangumiError> {
        throttle().await;

        let request = SearchRequest {
            keyword,
            sort: "match",
            filter: SearchFilter {
                subject_type: vec![2],
            },
        };

        let response = self
            .authorize(self.http.post(format!("{BASE_URL}/v0/search/subjects")))
            .query(&[("limit", "20")])
            .json(&request)
            .send()
            .await
            .map_err(map_network_error)?;

        let response = check_status(response)?;

        let body: SearchResponse = response.json().await.map_err(|error| {
            eprintln!("[AniShelf] Bangumi 搜索响应解析失败：{error}");
            BangumiError::Parse
        })?;

        Ok(body.data)
    }

    pub async fn subject(&self, subject_id: i64) -> Result<BangumiSubject, BangumiError> {
        throttle().await;

        let response = self
            .authorize(
                self.http
                    .get(format!("{BASE_URL}/v0/subjects/{subject_id}")),
            )
            .send()
            .await
            .map_err(map_network_error)?;

        let response = check_status(response)?;

        response.json().await.map_err(|error| {
            eprintln!("[AniShelf] Bangumi 作品响应解析失败：{error}");
            BangumiError::Parse
        })
    }

    fn authorize(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match self.token.as_deref() {
            Some(token) => request.bearer_auth(token),
            None => request,
        }
    }
}

async fn throttle() {
    let mutex = THROTTLE.get_or_init(|| Mutex::new(None));
    let mut last_request = mutex.lock().await;

    if let Some(last) = *last_request {
        let elapsed = last.elapsed();
        if elapsed < MIN_REQUEST_INTERVAL {
            tokio::time::sleep(MIN_REQUEST_INTERVAL - elapsed).await;
        }
    }

    *last_request = Some(Instant::now());
}

fn check_status(response: reqwest::Response) -> Result<reqwest::Response, BangumiError> {
    match response.status().as_u16() {
        404 => Err(BangumiError::NotFound),
        429 => Err(BangumiError::RateLimited),
        status if !(200..300).contains(&status) => Err(BangumiError::Http(status)),
        _ => Ok(response),
    }
}

fn map_network_error(error: reqwest::Error) -> BangumiError {
    eprintln!("[AniShelf] Bangumi 网络请求失败：{error}");
    BangumiError::Network
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUBJECT_JSON: &str = r#"{
        "id": 400602,
        "name": "葬送のフリーレン",
        "name_cn": "葬送的芙莉莲",
        "summary": "魔法使的旅途",
        "date": "2023-09-29",
        "eps": 28,
        "images": {
            "large": "https://example.com/large.jpg",
            "common": "",
            "medium": "https://example.com/medium.jpg"
        }
    }"#;

    #[test]
    fn parses_subject_and_derives_fields() {
        let subject: BangumiSubject = serde_json::from_str(SUBJECT_JSON).expect("subject");

        assert_eq!(subject.display_title(), "葬送的芙莉莲");
        assert_eq!(
            subject.original_title().as_deref(),
            Some("葬送のフリーレン")
        );
        assert_eq!(
            subject.cover_url().as_deref(),
            Some("https://example.com/large.jpg")
        );
        assert_eq!(
            subject.year_and_season(),
            (Some(2023), Some("summer".to_string()))
        );
        assert_eq!(subject.total_episodes(), Some(28));
    }

    #[test]
    fn parses_search_response() {
        let json = format!(r#"{{"data":[{SUBJECT_JSON}],"total":1}}"#);
        let response: SearchResponse = serde_json::from_str(&json).expect("search response");
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].id, 400602);
    }

    #[test]
    fn missing_optional_fields_are_tolerated() {
        let subject: BangumiSubject =
            serde_json::from_str(r#"{"id":1,"name":"Test"}"#).expect("minimal subject");

        assert_eq!(subject.display_title(), "Test");
        assert!(subject.original_title().is_none());
        assert!(subject.cover_url().is_none());
        assert_eq!(subject.year_and_season(), (None, None));
        assert!(subject.total_episodes().is_none());
    }

    #[test]
    fn seasons_follow_quarter_boundaries() {
        assert_eq!(season_for_month(1), Some("winter"));
        assert_eq!(season_for_month(3), Some("winter"));
        assert_eq!(season_for_month(4), Some("spring"));
        assert_eq!(season_for_month(7), Some("summer"));
        assert_eq!(season_for_month(10), Some("fall"));
        assert_eq!(season_for_month(13), None);
    }
}
