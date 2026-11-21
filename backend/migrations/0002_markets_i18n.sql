-- Thị trường (quốc gia + tiền tệ) và bản dịch nội dung món ăn.

-- Giá tham khảo của một khẩu phần ăn ngoài tại từng thị trường, theo tiền tệ địa phương.
-- Không có dòng cho một thị trường = món gần như không bán ở đó.
CREATE TABLE food_prices (
    food_id   UUID             NOT NULL REFERENCES foods (id) ON DELETE CASCADE,
    market    TEXT             NOT NULL CHECK (market IN ('VN', 'JP', 'KR', 'CN', 'GB')),
    currency  TEXT             NOT NULL,
    price_min DOUBLE PRECISION NOT NULL CHECK (price_min >= 0),
    price_max DOUBLE PRECISION NOT NULL CHECK (price_max >= price_min),
    PRIMARY KEY (food_id, market)
);

CREATE INDEX food_prices_market_idx ON food_prices (market, price_min, price_max);

-- Giá VND hiện có trở thành giá của thị trường VN.
INSERT INTO food_prices (food_id, market, currency, price_min, price_max)
SELECT id, 'VN', 'VND', price_min, price_max FROM foods
ON CONFLICT DO NOTHING;

-- Bản dịch nội dung món. Tiếng Việt là bản gốc nằm ngay trong bảng foods.
CREATE TABLE food_translations (
    food_id       UUID   NOT NULL REFERENCES foods (id) ON DELETE CASCADE,
    locale        TEXT   NOT NULL CHECK (locale IN ('en', 'ja', 'ko', 'zh')),
    name          TEXT   NOT NULL,
    description   TEXT   NOT NULL DEFAULT '',
    origin        TEXT   NOT NULL DEFAULT '',
    cooking_style TEXT   NOT NULL DEFAULT '',
    serving_note  TEXT   NOT NULL DEFAULT '',
    ingredients   JSONB  NOT NULL DEFAULT '[]'::JSONB,
    steps         JSONB  NOT NULL DEFAULT '[]'::JSONB,
    tags          TEXT[] NOT NULL DEFAULT '{}',
    PRIMARY KEY (food_id, locale)
);

-- Quán đã lưu: ghi lại thị trường để hiển thị đúng nhóm.
ALTER TABLE favorite_places ADD COLUMN market TEXT;
