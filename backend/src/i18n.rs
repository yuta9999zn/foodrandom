//! Ngôn ngữ và thị trường (quốc gia + tiền tệ).

use serde::Serialize;
use utoipa::ToSchema;

use crate::error::{AppError, AppResult};

/// Ngôn ngữ giao diện và nội dung. `vi` là bản gốc.
pub const LOCALES: [&str; 5] = ["vi", "en", "ja", "ko", "zh"];
pub const DEFAULT_LOCALE: &str = "vi";

pub fn parse_locale(value: Option<&str>) -> AppResult<&'static str> {
    match value.map(|v| v.trim().to_ascii_lowercase()) {
        None => Ok(DEFAULT_LOCALE),
        Some(v) if v.is_empty() => Ok(DEFAULT_LOCALE),
        Some(v) => {
            // Chấp nhận cả dạng "ja-JP", "zh-CN".
            let base = v.split(['-', '_']).next().unwrap_or("");
            LOCALES
                .iter()
                .find(|l| **l == base)
                .copied()
                .ok_or_else(|| {
                    AppError::validation(
                        "lang",
                        format!(
                            "Ngôn ngữ '{v}' không hỗ trợ. Cho phép: {}",
                            LOCALES.join(", ")
                        ),
                    )
                })
        }
    }
}

/// Một mức ngân sách của thị trường. `max = None` = không giới hạn trên.
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
pub struct BudgetBand {
    pub code: &'static str,
    pub min: f64,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
pub struct Market {
    /// Mã quốc gia ISO 3166-1 alpha-2.
    pub code: &'static str,
    /// Mã tiền tệ ISO 4217.
    pub currency: &'static str,
    /// Ngôn ngữ chính của thị trường, dùng khi tìm quán bằng Google Maps.
    pub language: &'static str,
    /// Tâm bản đồ mặc định (thủ đô / thành phố lớn).
    pub center_lat: f64,
    pub center_lng: f64,
    pub budgets: [BudgetBand; 5],
}

const fn band(code: &'static str, min: f64, max: Option<f64>) -> BudgetBand {
    BudgetBand { code, min, max }
}

pub const MARKETS: [Market; 5] = [
    Market {
        code: "VN",
        currency: "VND",
        language: "vi",
        center_lat: 10.7769,
        center_lng: 106.7009,
        budgets: [
            band("b1", 0.0, Some(30_000.0)),
            band("b2", 30_000.0, Some(50_000.0)),
            band("b3", 50_000.0, Some(100_000.0)),
            band("b4", 100_000.0, Some(200_000.0)),
            band("b5", 200_000.0, None),
        ],
    },
    Market {
        code: "JP",
        currency: "JPY",
        language: "ja",
        center_lat: 35.6812,
        center_lng: 139.7671,
        budgets: [
            band("b1", 0.0, Some(800.0)),
            band("b2", 800.0, Some(1_500.0)),
            band("b3", 1_500.0, Some(3_000.0)),
            band("b4", 3_000.0, Some(6_000.0)),
            band("b5", 6_000.0, None),
        ],
    },
    Market {
        code: "KR",
        currency: "KRW",
        language: "ko",
        center_lat: 37.5665,
        center_lng: 126.9780,
        budgets: [
            band("b1", 0.0, Some(8_000.0)),
            band("b2", 8_000.0, Some(15_000.0)),
            band("b3", 15_000.0, Some(30_000.0)),
            band("b4", 30_000.0, Some(60_000.0)),
            band("b5", 60_000.0, None),
        ],
    },
    Market {
        code: "CN",
        currency: "CNY",
        language: "zh",
        center_lat: 31.2304,
        center_lng: 121.4737,
        budgets: [
            band("b1", 0.0, Some(25.0)),
            band("b2", 25.0, Some(50.0)),
            band("b3", 50.0, Some(100.0)),
            band("b4", 100.0, Some(200.0)),
            band("b5", 200.0, None),
        ],
    },
    Market {
        code: "GB",
        currency: "GBP",
        language: "en",
        center_lat: 51.5074,
        center_lng: -0.1278,
        budgets: [
            band("b1", 0.0, Some(8.0)),
            band("b2", 8.0, Some(15.0)),
            band("b3", 15.0, Some(30.0)),
            band("b4", 30.0, Some(60.0)),
            band("b5", 60.0, None),
        ],
    },
];

pub const DEFAULT_MARKET: &str = "VN";

pub fn market(code: &str) -> Option<&'static Market> {
    MARKETS.iter().find(|m| m.code == code)
}

pub fn parse_market(value: Option<&str>) -> AppResult<&'static Market> {
    let code = value
        .map(|v| v.trim().to_ascii_uppercase())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_MARKET.into());
    market(&code).ok_or_else(|| {
        AppError::validation(
            "market",
            format!(
                "Thị trường '{code}' không hỗ trợ. Cho phép: {}",
                MARKETS
                    .iter()
                    .map(|m| m.code)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        )
    })
}

pub const BUDGET_CODES: [&str; 5] = ["b1", "b2", "b3", "b4", "b5"];

impl Market {
    pub fn band(&self, code: &str) -> Option<BudgetBand> {
        self.budgets.iter().find(|b| b.code == code).copied()
    }
}

/// Cụm từ "nhà hàng món <nền ẩm thực>" theo ngôn ngữ, dùng làm truy vấn Google Places khi không có tên món.
pub fn cuisine_restaurant_query(cuisine: &str, language: &str) -> String {
    let (vi, en, ja, ko, zh) = match cuisine {
        "vietnam" => (
            "nhà hàng món Việt",
            "Vietnamese restaurant",
            "ベトナム料理",
            "베트남 음식점",
            "越南餐厅",
        ),
        "japan" => (
            "nhà hàng Nhật",
            "Japanese restaurant",
            "和食 レストラン",
            "일식당",
            "日本料理",
        ),
        "china" => (
            "nhà hàng Trung Hoa",
            "Chinese restaurant",
            "中華料理",
            "중식당",
            "中餐厅",
        ),
        "korea" => (
            "nhà hàng Hàn Quốc",
            "Korean restaurant",
            "韓国料理",
            "한식당",
            "韩国料理",
        ),
        _ => (
            "nhà hàng Anh",
            "British restaurant",
            "イギリス料理",
            "영국 음식점",
            "英式餐厅",
        ),
    };
    match language {
        "en" => en,
        "ja" => ja,
        "ko" => ko,
        "zh" => zh,
        _ => vi,
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_locale_variants() {
        assert_eq!(parse_locale(None).unwrap(), "vi");
        assert_eq!(parse_locale(Some("ja-JP")).unwrap(), "ja");
        assert_eq!(parse_locale(Some("ZH")).unwrap(), "zh");
        assert!(parse_locale(Some("fr")).is_err());
    }

    #[test]
    fn markets_have_ordered_bands() {
        for m in MARKETS {
            for pair in m.budgets.windows(2) {
                assert_eq!(pair[0].max, Some(pair[1].min), "{}", m.code);
            }
            assert!(parse_market(Some(&m.code.to_lowercase())).is_ok());
        }
        assert!(parse_market(Some("US")).is_err());
    }
}
