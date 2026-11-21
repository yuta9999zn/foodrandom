/// Bỏ dấu tiếng Việt và chuyển về chữ thường, dùng cho tìm kiếm không dấu.
pub fn fold_vi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars().flat_map(|c| c.to_lowercase()) {
        let mapped = match ch {
            'à' | 'á' | 'ạ' | 'ả' | 'ã' | 'â' | 'ầ' | 'ấ' | 'ậ' | 'ẩ' | 'ẫ' | 'ă' | 'ằ' | 'ắ' | 'ặ' | 'ẳ' | 'ẵ' => 'a',
            'è' | 'é' | 'ẹ' | 'ẻ' | 'ẽ' | 'ê' | 'ề' | 'ế' | 'ệ' | 'ể' | 'ễ' => 'e',
            'ì' | 'í' | 'ị' | 'ỉ' | 'ĩ' => 'i',
            'ò' | 'ó' | 'ọ' | 'ỏ' | 'õ' | 'ô' | 'ồ' | 'ố' | 'ộ' | 'ổ' | 'ỗ' | 'ơ' | 'ờ' | 'ớ' | 'ợ' | 'ở' | 'ỡ' => 'o',
            'ù' | 'ú' | 'ụ' | 'ủ' | 'ũ' | 'ư' | 'ừ' | 'ứ' | 'ự' | 'ử' | 'ữ' => 'u',
            'ỳ' | 'ý' | 'ỵ' | 'ỷ' | 'ỹ' => 'y',
            'đ' => 'd',
            // Dấu thanh dạng tổ hợp (Unicode NFD) thì bỏ.
            '\u{0300}' | '\u{0301}' | '\u{0303}' | '\u{0309}' | '\u{0323}' | '\u{0302}' | '\u{0306}' | '\u{031B}' => continue,
            other => other,
        };
        out.push(mapped);
    }
    out
}

/// Khoảng cách đường chim bay giữa hai toạ độ (km), công thức haversine.
pub fn haversine_km(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    const EARTH_RADIUS_KM: f64 = 6371.0088;
    let (phi1, phi2) = (lat1.to_radians(), lat2.to_radians());
    let d_phi = (lat2 - lat1).to_radians();
    let d_lambda = (lng2 - lng1).to_radians();
    let a = (d_phi / 2.0).sin().powi(2) + phi1.cos() * phi2.cos() * (d_lambda / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS_KM * a.sqrt().asin()
}

pub fn valid_coords(lat: f64, lng: f64) -> bool {
    lat.is_finite() && lng.is_finite() && (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lng)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_vi_strips_diacritics() {
        assert_eq!(fold_vi("Phở bò Nam Định"), "pho bo nam dinh");
        assert_eq!(fold_vi("Bánh canh ghẹ"), "banh canh ghe");
        assert_eq!(fold_vi("ĐẬU HŨ"), "dau hu");
    }

    #[test]
    fn haversine_known_distance() {
        // Bến Thành → Nhà thờ Đức Bà, khoảng 0,9 km.
        let d = haversine_km(10.7725, 106.6980, 10.7798, 106.6990);
        assert!((0.7..1.1).contains(&d), "{d}");
    }
}
