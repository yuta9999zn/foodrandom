# FoodRandom — Hôm nay ăn gì?

Website giúp chọn món ăn: lọc theo bữa, loại món, phong cách, món nước, ngân sách và vị trí, rồi để hệ thống **random** một món trong 12 món đang hiển thị. Có 100 món tiêu biểu của 5 nền ẩm thực (Việt Nam, Nhật Bản, Trung Quốc, Hàn Quốc, Anh), mỗi món có ảnh, công thức, nguyên liệu, calo và dinh dưỡng.

- **5 ngôn ngữ:** Tiếng Việt, English, 日本語, 한국어, 中文. Bản dịch gồm cả giao diện lẫn nội dung món.
- **Chọn quốc gia trước:** VN, JP, KR, CN, GB. Giá và mức ngân sách theo tiền tệ nơi bạn ở (VND, JPY, KRW, CNY, GBP).
- **Không cần đăng nhập.** Mọi tính năng dùng được ngay. Món yêu thích lưu trên trình duyệt (localStorage).
- **Tìm quán bằng Google Maps.** Chọn một món rồi bấm *Tìm trên Google Maps*: Google Maps mở ra (tab mới, hoặc ứng dụng trên điện thoại) với tên món trong ô tìm kiếm, người dùng tự tìm và chọn quán gần mình nhất. Không cần API key.

**Mục lục:** [Chạy nhanh](#chạy-trên-máy-lập-trình) · [Lấy API key Google Maps](#lấy-api-key-google-maps) · [API](#api-chính) · [Triển khai production](#triển-khai-production-và-chịu-tải) · [Nguồn dữ liệu](#nguồn-dữ-liệu)

| Thư mục | Nội dung |
|---|---|
| `backend/` | API Rust: Axum, Tokio, SQLx (PostgreSQL), Redis, utoipa (OpenAPI) |
| `backend/seed/` | Dữ liệu món (`foods/`), bản dịch (`i18n/<lang>/`), giá theo thị trường (`prices/`) |
| `frontend/` | Next.js 15 (App Router), TypeScript, Tailwind, shadcn/ui (Radix), TanStack Query, Zustand, React Hook Form + Zod, Framer Motion |
| `infra/nginx/` | Cấu hình Nginx cho production |
| `design/` | Ảnh thiết kế trang chủ và banner gốc |

## Chạy trên máy lập trình

Cần cài sẵn:
- [Docker Desktop](https://www.docker.com/products/docker-desktop/) (để chạy PostgreSQL, Redis);
- [Rust](https://rustup.rs/) bản stable;
- [Node.js 22](https://nodejs.org/).

```bash
git clone https://github.com/yuta9999zn/foodrandom.git
cd foodrandom

# 1. Hạ tầng: PostgreSQL (54331), Redis (63791)
docker compose -f docker-compose.dev.yml up -d

# 2. Backend — http://127.0.0.1:8180, tài liệu API http://127.0.0.1:8180/docs
cd backend
cp .env.example .env
cargo run            # tự chạy migration và nạp 100 món + bản dịch + giá

# 3. Frontend — http://localhost:3100
cd ../frontend
cp .env.example .env.local
npm install
npm run dev
```

> **Windows:** nếu đường dẫn dự án có dấu tiếng Việt, trình liên kết MinGW không ghi được file trong `target/`. Đặt thư mục build sang đường dẫn không dấu, ví dụ `set CARGO_TARGET_DIR=C:\foodrandom-target`.
>
> Nếu `localhost` phân giải sang IPv6 và kết nối Docker bị treo, hãy dùng `127.0.0.1` trong `.env` (mặc định đã vậy).

Mở http://localhost:3100.
Lần đầu vào, trang hỏi bạn đang ở quốc gia nào để hiển thị giá đúng tiền tệ.

Ứng dụng chạy được ngay **không cần API key Google**. Nút *Tìm trên Google Maps* chỉ là một liên kết nên không cần key. Key chỉ dùng cho bộ lọc "Địa điểm gần bạn" ở trang chủ (gắn khoảng cách tới quán gần nhất cho từng món): không có key thì dùng OpenStreetMap, tối đa 10 km. Muốn dùng Google, xem [Lấy API key Google Maps](#lấy-api-key-google-maps).

### Lấy API key Google Maps

FoodRandom dùng 2 API của [Google Maps Platform](https://mapsplatform.google.com/maps-products/):

| API | Dùng để |
|---|---|
| **Places API (New)** | Tìm quán quanh vị trí người dùng, kèm số sao, số lượt đánh giá và trạng thái mở cửa |
| **Geocoding API** | Đổi địa chỉ người dùng nhập tay thành toạ độ, và ngược lại |

Nút "Tìm trên Google Maps" mở thẳng Google Maps nên **không** cần key.

1. **Tạo project.**
   - Vào https://console.cloud.google.com/ và đăng nhập bằng tài khoản Google.
   - Bấm chọn project ở thanh trên cùng → **New Project** → đặt tên, ví dụ `FoodRandom` → **Create**.
2. **Bật thanh toán (Billing).**
   - Vào menu **Billing** → **Link a billing account**, tạo hoặc chọn tài khoản thanh toán và nhập thẻ.
   - Google bắt buộc bước này mới cho bật Maps API. Mỗi API có hạn mức dùng miễn phí hằng tháng; xem bảng giá hiện hành tại https://mapsplatform.google.com/pricing/.
3. **Bật 2 API.**
   - Vào **APIs & Services → Library**.
   - Tìm **Places API (New)** → **Enable**. Lưu ý chọn đúng bản *New*, không phải "Places API" cũ.
   - Tìm **Geocoding API** → **Enable**.
4. **Tạo API key.** Vào **APIs & Services → Credentials → Create credentials → API key**, rồi sao chép key vừa tạo.
5. **Giới hạn key** (rất nên làm để key không bị lạm dụng). Bấm vào key vừa tạo:
   - **API restrictions** → *Restrict key* → chỉ tick **Places API (New)** và **Geocoding API**.
   - **Application restrictions:**
     - Khi chạy thử trên máy, có thể để *None*.
     - Khi đưa lên server, chọn **IP addresses** và nhập IP công khai của server.
   - Key chỉ được dùng ở backend (máy chủ) và không bao giờ gửi xuống trình duyệt, nên **không** chọn kiểu giới hạn *HTTP referrers*.
6. **Đặt cảnh báo chi phí** (khuyến nghị): **Billing → Budgets & alerts → Create budget**, ví dụ báo khi vượt 10 USD/tháng.
7. **Điền key vào FoodRandom.** Mở file `backend/.env` (bản production là `.env.production`) và sửa:
   ```env
   LOCATION_PROVIDER=auto
   GOOGLE_MAPS_API_KEY=AIza...key-của-bạn
   ```
   Sau đó khởi động lại backend (`Ctrl+C` rồi `cargo run`).
8. **Kiểm tra.**
   - Mở http://127.0.0.1:8180/api/v1/location/provider. Nếu thấy `"provider":"google"` là đã chạy bằng Google.
   - Ở trang chủ, nhập địa chỉ ở ô *Địa điểm gần bạn*. Các thẻ món sẽ hiện tên và khoảng cách tới quán gần nhất.

> ⚠️ **Không commit API key lên GitHub.** Các file `backend/.env`, `.env.production` và `frontend/.env.local` đã nằm trong `.gitignore`. Nếu lỡ để lộ key, vào *Credentials* xoá key cũ và tạo key mới.

**Lỗi thường gặp**

| Hiện tượng | Cách xử lý |
|---|---|
| `"provider":"osm"` dù đã điền key | Kiểm tra key nằm trong `backend/.env` và backend đã được khởi động lại |
| Log backend báo `PERMISSION_DENIED` / `API_KEY_INVALID` | Chưa bật **Places API (New)**, hoặc key bị giới hạn sai API/IP |
| Log báo `REQUEST_DENIED` / billing | Project chưa liên kết tài khoản thanh toán |

### Địa điểm và Google Maps

| `LOCATION_PROVIDER` | Nguồn dữ liệu |
|---|---|
| `auto` (mặc định) | Có `GOOGLE_MAPS_API_KEY` thì dùng Google, không thì dùng OpenStreetMap |
| `google` | Google Maps Platform: Places API (New) Text Search và Geocoding API. Có điểm đánh giá của Google. |
| `osm` | OpenStreetMap: Overpass tìm quán, Nominatim đổi địa chỉ. Dữ liệu thật, không cần key, không có đánh giá. |
| `mock` | **Dữ liệu giả lập**, chỉ để phát triển giao diện khi không có mạng. Quán có tên "[GIẢ LẬP]" và giao diện hiện cảnh báo. Không được bật ở production. |

- **Tìm quán:** nút *Tìm trên Google Maps* (thẻ món, kết quả random, trang chi tiết) mở `https://www.google.com/maps/search/?api=1&query=<tên món>` theo định dạng [Maps URLs](https://developers.google.com/maps/documentation/urls/get-started) chính thức. Google Maps tự dùng vị trí của người dùng; người dùng chọn quán gần nhất ngay trong Google Maps.
- Khoảng cách ở bộ lọc trang chủ luôn tính từ toạ độ thật của người dùng (Browser Geolocation hoặc địa chỉ nhập tay). Ứng dụng không bịa toạ độ, địa chỉ hay điểm đánh giá.
- `GET /api/v1/location/nearby` vẫn còn trong API (xếp quán theo số sao Google khi có key) nhưng giao diện không dùng nữa.

## API chính

Tài liệu OpenAPI/Swagger: `GET /docs`, đặc tả JSON: `GET /api-docs/openapi.json`.

| Phương thức | Đường dẫn | Mô tả |
|---|---|---|
| GET | `/api/v1/foods?market=JP&lang=ja&meals=dinner&budgets=b2&page=1` | 12 món/trang theo bộ lọc. Thiếu món khớp thì lấp bằng "gợi ý mở rộng" cho đủ 12. |
| POST | `/api/v1/random` | Random **ở backend** trong 12 món của trang đang xem (`scope=page`) hoặc mọi món khớp (`scope=all`). Có trọng số, tránh lặp món vừa chọn. |
| GET | `/api/v1/foods/{slug}?lang=&market=` | Chi tiết: mô tả, nguồn gốc, công thức, dinh dưỡng, món tương tự |
| GET | `/api/v1/location/nearby?lat=&lng=&food=pho-bo&market=JP&sort=rating` | Vài quán gần nhất phục vụ món đó; `sort=rating` xếp theo số sao Google, `sort=distance` theo khoảng cách |
| GET | `/api/v1/location/geocode?q=&market=` | Địa chỉ nhập tay → toạ độ, giới hạn trong quốc gia đã chọn |

API công khai, không có đăng nhập.

## Triển khai production và chịu tải

```bash
cp .env.production.example .env.production   # điền PUBLIC_URL, mật khẩu DB, (tuỳ chọn) GOOGLE_MAPS_API_KEY
docker compose --env-file .env.production up -d --build
# Tải tăng: thêm bản sao, Nginx tự cân bằng tới mọi bản sao
docker compose --env-file .env.production up -d --scale backend=6 --scale frontend=4
```

Các lớp giúp hệ thống không sập khi nhiều người truy cập:

1. **Backend không trạng thái.**
   - Không có phiên đăng nhập. Giới hạn tần suất và cache nằm ở Redis.
   - Vì vậy có thể chạy nhiều bản sao sau Nginx (`least_conn`, tự bỏ qua bản sao lỗi).
2. **Bảo vệ quá tải trong từng bản sao** (Tower):
   - Giới hạn số request đồng thời. Quá ngưỡng thì *load-shed* trả 503 ngay thay vì xếp hàng vô hạn.
   - Timeout 20 giây, giới hạn kích thước body, bắt panic.
   - Pool kết nối PostgreSQL có giới hạn.
3. **Cache nhiều tầng:**
   - Nginx cache 30 giây cho API danh sách/chi tiết món của khách, có `proxy_cache_lock` để chống "dồn request" khi cache hết hạn.
   - Ảnh và tài sản tĩnh được cache dài hạn.
   - Redis cache kết quả lọc món (60 giây), chi tiết món (5 phút), kết quả địa điểm (15 phút) và geocode (24 giờ).
4. **Tra địa điểm không chặn danh sách.**
   - Danh sách món trả ngay. Khoảng cách được tra ở nền, mỗi ô toạ độ chỉ một bản sao làm nhờ khoá Redis, rồi client hỏi lại.
   - Dịch vụ bản đồ chậm không làm chậm trang chủ.
5. **Giới hạn tần suất:**
   - Nginx: 30 request/giây mỗi IP cho API. Khi Nginx đứng sau một reverse proxy khác (mạng nội bộ), IP thật lấy từ `X-Forwarded-For`.
   - Backend thêm giới hạn riêng cho tra địa điểm.
6. **Frontend Next.js chạy dạng standalone**, có nhiều bản sao. Ảnh được tối ưu sang AVIF/WebP theo kích thước màn hình.

Khi vượt quá một máy, các image này chạy được trên Kubernetes/ECS. Lúc đó dùng PostgreSQL và Redis managed, thêm read replica cho PostgreSQL và CDN trước Nginx.

## Kiểm thử

```bash
cd backend && cargo test        # unit test: lọc ngân sách, phân trang 12 món, random có trọng số, i18n…
cd frontend && npm run typecheck && npm run lint
```

CI GitHub Actions (`.github/workflows/ci.yml`) chạy các bước sau:
- clippy và test cho backend;
- kiểm tra dữ liệu seed: đủ ảnh, đủ bản dịch, đủ giá;
- typecheck, lint và build cho frontend;
- build Docker image.

## Nguồn dữ liệu

> Ảnh món ăn trong `frontend/public/images/foods/` thuộc về tác giả của từng ảnh trên Wikimedia Commons và được dùng theo giấy phép ghi trong `backend/seed/foods/*.json` (trường `image.author`, `image.license`, `image.source_url`). Khi tái sử dụng ảnh, hãy giữ nguyên ghi nguồn theo giấy phép đó.

- **Ảnh món ăn:** Wikimedia Commons. Giấy phép CC0, CC BY, CC BY-SA hoặc KOGL. Tác giả và giấy phép ghi ở trang chi tiết từng món.
- **Giá:** số tham khảo cho một khẩu phần ăn ngoài tại từng quốc gia, ước tính, không phải giá của một quán cụ thể.
- **Calo và dinh dưỡng:** ước tính theo khẩu phần phổ biến.
- **Trợ lý AI:** đã có điểm vào trên giao diện (nút "Trợ lý AI"), tính năng sẽ triển khai sau.
