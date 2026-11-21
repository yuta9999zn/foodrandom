// Liên kết Google Maps theo định dạng Maps URLs chính thức — mở được trên web và ứng dụng, không cần API key.
// https://developers.google.com/maps/documentation/urls/get-started

export interface LatLng {
  lat: number;
  lng: number;
}

const coord = (p: LatLng) => `${p.lat.toFixed(6)},${p.lng.toFixed(6)}`;

/** Chỉ đường từ vị trí người dùng (nếu có) tới một toạ độ. Thiếu điểm đi thì Google Maps dùng vị trí hiện tại. */
export function directionsUrl(destination: LatLng, origin?: LatLng | null, destinationName?: string) {
  const params = new URLSearchParams({ api: '1', destination: coord(destination), travelmode: 'driving' });
  if (origin) params.set('origin', coord(origin));
  if (destinationName) params.set('destination', `${destinationName} @${coord(destination)}`);
  // Giữ toạ độ làm đích để tuyến đường luôn chính xác dù tên quán trùng nhau.
  params.set('destination', coord(destination));
  return `https://www.google.com/maps/dir/?${params.toString()}`;
}

/** Mức thu phóng Google Maps tương ứng bán kính tìm (km): bán kính càng lớn bản đồ càng thu nhỏ. */
function zoomFor(radiusKm?: number | null) {
  if (radiusKm == null) return 12;
  if (radiusKm <= 1) return 16;
  if (radiusKm <= 2) return 15;
  if (radiusKm <= 5) return 14;
  if (radiusKm <= 10) return 13;
  if (radiusKm <= 20) return 12;
  return 11;
}

/** Tìm một món ăn (hoặc quán) trên Google Maps, quanh vị trí người dùng và trong phạm vi bán kính đã chọn. */
export function searchUrl(query: string, near?: LatLng | null, radiusKm?: number | null) {
  const encoded = encodeURIComponent(query);
  if (near) return `https://www.google.com/maps/search/${encoded}/@${coord(near)},${zoomFor(radiusKm)}z`;
  return `https://www.google.com/maps/search/?api=1&query=${encoded}`;
}
