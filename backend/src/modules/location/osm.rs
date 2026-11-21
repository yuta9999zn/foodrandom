use std::time::Duration;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

use super::{
    dto::{GeoPoint, RawPlace},
    provider::{NearbySearch, PlaceProvider, CUISINES},
};
use crate::util::fold_vi;

/// Giá trị thẻ `cuisine` của OpenStreetMap ứng với từng nền ẩm thực của FoodRandom.
const OSM_CUISINE_TAGS: [(&str, &[&str]); 5] = [
    ("vietnam", &["vietnamese", "pho", "banh_mi", "bun_cha", "bun", "com_tam", "hu_tieu"]),
    ("japan", &["japanese", "sushi", "ramen", "udon", "soba", "tempura", "okonomiyaki", "yakitori", "donburi"]),
    ("china", &["chinese", "dim_sum", "cantonese", "sichuan", "hotpot", "dumpling", "hong_kong"]),
    ("korea", &["korean", "korean_bbq", "bibimbap"]),
    ("uk", &["british", "english", "fish_and_chips", "scottish"]),
];

/// Nhà cung cấp địa điểm dùng dữ liệu mở OpenStreetMap: Overpass (tìm quán) và Nominatim (địa chỉ ⇄ toạ độ).
/// Dữ liệu là thật, không cần API key. OpenStreetMap không có điểm đánh giá nên `rating` luôn rỗng.
pub struct OsmProvider {
    http: reqwest::Client,
    /// Danh sách máy chủ Overpass; máy chủ đầu lỗi/chậm thì thử máy chủ kế tiếp.
    overpass_urls: Vec<String>,
    nominatim_url: String,
}

impl OsmProvider {
    pub fn new(overpass_urls: &[String], nominatim_url: &str, user_agent: &str) -> Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(user_agent)
            // Mỗi máy chủ tối đa 8 giây để còn thời gian thử máy chủ dự phòng trong ngân sách 17 giây.
            .timeout(Duration::from_secs(8))
            .connect_timeout(Duration::from_secs(4))
            .build()
            .context("không tạo được HTTP client cho OpenStreetMap")?;
        if overpass_urls.is_empty() {
            anyhow::bail!("OVERPASS_URL không được để trống");
        }
        Ok(Self {
            http,
            overpass_urls: overpass_urls.to_vec(),
            nominatim_url: nominatim_url.trim_end_matches('/').to_string(),
        })
    }

    async fn overpass(&self, query: &str) -> Result<OverpassResponse> {
        let mut last_error = anyhow::anyhow!("chưa gọi Overpass");
        for url in &self.overpass_urls {
            let attempt = async {
                let response = self.http.post(url).form(&[("data", query)]).send().await.context("không gọi được Overpass API")?;
                if !response.status().is_success() {
                    bail!("Overpass API {url} trả mã {}", response.status());
                }
                response.json::<OverpassResponse>().await.context("phản hồi Overpass không đọc được")
            };
            match attempt.await {
                Ok(body) => return Ok(body),
                Err(err) => {
                    tracing::warn!(%url, error = %err, "máy chủ Overpass lỗi, thử máy chủ khác");
                    last_error = err;
                }
            }
        }
        Err(last_error)
    }
}

#[derive(Deserialize)]
struct OverpassResponse {
    #[serde(default)]
    elements: Vec<OverpassElement>,
}

#[derive(Deserialize)]
struct OverpassElement {
    #[serde(rename = "type")]
    kind: String,
    id: i64,
    lat: Option<f64>,
    lon: Option<f64>,
    center: Option<OverpassCenter>,
    #[serde(default)]
    tags: std::collections::HashMap<String, String>,
}

#[derive(Deserialize)]
struct OverpassCenter {
    lat: f64,
    lon: f64,
}

/// Chỉ giữ chữ, số và khoảng trắng để chèn an toàn vào biểu thức chính quy của Overpass.
fn sanitize_keyword(keyword: &str) -> String {
    keyword
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ')
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn cuisines_from_tag(tag: &str) -> Vec<String> {
    let tokens: Vec<String> = tag.split(';').map(|t| t.trim().to_ascii_lowercase()).collect();
    OSM_CUISINE_TAGS
        .iter()
        .filter(|(_, values)| tokens.iter().any(|t| values.contains(&t.as_str())))
        .map(|(code, _)| code.to_string())
        .collect()
}

fn address_from_tags(tags: &std::collections::HashMap<String, String>) -> Option<String> {
    if let Some(full) = tags.get("addr:full") {
        return Some(full.clone());
    }
    let street = match (tags.get("addr:housenumber"), tags.get("addr:street")) {
        (Some(no), Some(street)) => Some(format!("{no} {street}")),
        (None, Some(street)) => Some(street.clone()),
        _ => None,
    };
    let parts: Vec<String> = [
        street,
        tags.get("addr:subdistrict").cloned().or_else(|| tags.get("addr:suburb").cloned()),
        tags.get("addr:district").cloned(),
        tags.get("addr:city").cloned().or_else(|| tags.get("addr:province").cloned()),
    ]
    .into_iter()
    .flatten()
    .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

#[async_trait]
impl PlaceProvider for OsmProvider {
    fn name(&self) -> &'static str {
        "osm"
    }

    fn attribution(&self) -> &'static str {
        "Dữ liệu địa điểm © những người đóng góp cho OpenStreetMap"
    }

    fn max_radius_m(&self) -> u32 {
        10_000
    }

    async fn nearby(&self, search: &NearbySearch) -> Result<Vec<RawPlace>> {
        let wanted: Vec<&str> = if search.cuisines.is_empty() {
            CUISINES.to_vec()
        } else {
            search.cuisines.iter().map(String::as_str).collect()
        };
        let tag_values: Vec<&str> = OSM_CUISINE_TAGS
            .iter()
            .filter(|(code, _)| wanted.contains(code))
            .flat_map(|(_, values)| values.iter().copied())
            .collect();
        if tag_values.is_empty() {
            return Ok(vec![]);
        }

        let around = format!("(around:{},{:.6},{:.6})", search.radius_m, search.lat, search.lng);
        let amenity = r#"["amenity"~"^(restaurant|fast_food|cafe|food_court|pub)$"]"#;
        let mut statements =
            format!("nwr{around}{amenity}[\"cuisine\"~\"(^|;) ?({})( ?;|$)\",i];", tag_values.join("|"));

        let keyword = search.keyword.as_deref().map(sanitize_keyword).filter(|k| k.chars().count() >= 2);
        if let Some(keyword) = &keyword {
            statements.push_str(&format!("nwr{around}{amenity}[\"name\"~\"{keyword}\",i];"));
        }
        let query = format!("[out:json][timeout:8];({statements});out center tags 200;");
        let body = self.overpass(&query).await?;

        let folded_keyword = keyword.as_deref().map(fold_vi);
        let mut places = Vec::with_capacity(body.elements.len());
        for element in body.elements {
            let Some(name) = element.tags.get("name:vi").or_else(|| element.tags.get("name")).cloned() else {
                continue;
            };
            let (lat, lng) = match (element.lat, element.lon, &element.center) {
                (Some(lat), Some(lon), _) => (lat, lon),
                (_, _, Some(center)) => (center.lat, center.lon),
                _ => continue,
            };
            let mut cuisines = element.tags.get("cuisine").map(|t| cuisines_from_tag(t)).unwrap_or_default();
            cuisines.retain(|c| wanted.contains(&c.as_str()));
            // Quán khớp theo tên món (ví dụ "Phở Hoà") được gán nền ẩm thực đang tìm.
            if cuisines.is_empty() {
                let name_matches = folded_keyword.as_deref().is_some_and(|k| fold_vi(&name).contains(k));
                if name_matches && wanted.len() == 1 {
                    cuisines.push(wanted[0].to_string());
                } else {
                    continue;
                }
            }
            places.push(RawPlace {
                place_ref: format!("{}/{}", element.kind, element.id),
                address: address_from_tags(&element.tags),
                name,
                lat,
                lng,
                cuisines,
                rating: None,
                rating_count: None,
                open_now: None,
                maps_url: None,
            });
        }
        Ok(places)
    }

    async fn geocode(&self, query: &str, country: &str, language: &str) -> Result<Vec<GeoPoint>> {
        let country = country.to_ascii_lowercase();
        let response = self
            .http
            .get(format!("{}/search", self.nominatim_url))
            .query(&[
                ("q", query),
                ("format", "jsonv2"),
                ("limit", "5"),
                ("countrycodes", country.as_str()),
                ("accept-language", language),
            ])
            .send()
            .await
            .context("không gọi được Nominatim")?;
        if !response.status().is_success() {
            bail!("Nominatim trả mã {}", response.status());
        }
        let items: Vec<Value> = response.json().await.context("phản hồi Nominatim không đọc được")?;
        Ok(items
            .iter()
            .filter_map(|item| {
                Some(GeoPoint {
                    label: item.get("display_name")?.as_str()?.to_string(),
                    lat: item.get("lat")?.as_str()?.parse().ok()?,
                    lng: item.get("lon")?.as_str()?.parse().ok()?,
                })
            })
            .collect())
    }

    async fn reverse(&self, lat: f64, lng: f64, language: &str) -> Result<Option<String>> {
        let response = self
            .http
            .get(format!("{}/reverse", self.nominatim_url))
            .query(&[
                ("lat", format!("{lat:.6}")),
                ("lon", format!("{lng:.6}")),
                ("format", "jsonv2".to_string()),
                ("zoom", "16".to_string()),
                ("accept-language", language.to_string()),
            ])
            .send()
            .await
            .context("không gọi được Nominatim")?;
        if !response.status().is_success() {
            bail!("Nominatim trả mã {}", response.status());
        }
        let body: Value = response.json().await.context("phản hồi Nominatim không đọc được")?;
        let Some(address) = body.get("address") else {
            return Ok(None);
        };
        let pick = |keys: &[&str]| keys.iter().find_map(|k| address.get(*k).and_then(Value::as_str)).map(str::to_string);
        // Nhãn ngắn kiểu "Phường Bến Nghé, Quận 1, Thành phố Hồ Chí Minh".
        let parts: Vec<String> = [
            pick(&["quarter", "suburb", "neighbourhood", "village", "town"]),
            pick(&["city_district", "district", "county"]),
            pick(&["city", "state", "province"]),
        ]
        .into_iter()
        .flatten()
        .collect();
        if parts.is_empty() {
            Ok(body.get("display_name").and_then(Value::as_str).map(str::to_string))
        } else {
            Ok(Some(parts.join(", ")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_osm_cuisine_tags() {
        assert_eq!(cuisines_from_tag("vietnamese;pho"), vec!["vietnam"]);
        assert_eq!(cuisines_from_tag("Japanese; korean"), vec!["japan", "korea"]);
        assert!(cuisines_from_tag("pizza").is_empty());
    }

    #[test]
    fn keyword_is_sanitized() {
        assert_eq!(sanitize_keyword("Phở\" ](bò)"), "Phở bò");
    }
}
