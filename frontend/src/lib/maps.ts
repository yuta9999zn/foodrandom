// Liên kết Google Maps theo định dạng Maps URLs chính thức — mở được trên web và ứng dụng, không cần API key.
// https://developers.google.com/maps/documentation/urls/get-started#search-action

/**
 * Mở Google Maps với tên món trong ô tìm kiếm. Google Maps tự dùng vị trí hiện tại của người dùng
 * (ứng dụng trên điện thoại hoặc trình duyệt) nên người dùng chỉ việc chọn quán gần mình nhất.
 */
export function mapsSearchUrl(query: string) {
  return `https://www.google.com/maps/search/?api=1&query=${encodeURIComponent(query.trim())}`;
}
