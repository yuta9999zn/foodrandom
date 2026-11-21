-- FoodRandom — lược đồ khởi tạo
-- Món ăn, người dùng, đánh giá, món yêu thích, quán yêu thích.

CREATE TABLE foods (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug             TEXT        NOT NULL UNIQUE,
    name             TEXT        NOT NULL,
    original_name    TEXT,
    cuisine          TEXT        NOT NULL CHECK (cuisine IN ('vietnam', 'japan', 'china', 'korea', 'uk')),
    style            TEXT        NOT NULL CHECK (style IN ('traditional', 'modern')),
    dish_type        TEXT        NOT NULL CHECK (dish_type IN ('main', 'side', 'fast_food', 'snack', 'dessert', 'drink')),
    meals            TEXT[]      NOT NULL CHECK (meals <@ ARRAY['breakfast', 'lunch', 'dinner', 'snack']::TEXT[] AND cardinality(meals) > 0),
    soup_type        TEXT        NOT NULL CHECK (soup_type IN ('pho', 'bun', 'mi', 'hu_tieu', 'banh_canh', 'lau', 'kho', 'khac')),
    price_min        INTEGER     NOT NULL CHECK (price_min >= 0),
    price_max        INTEGER     NOT NULL CHECK (price_max >= price_min),
    calories_kcal    INTEGER     NOT NULL CHECK (calories_kcal >= 0),
    serving_note     TEXT        NOT NULL DEFAULT '',
    protein_g        REAL        NOT NULL DEFAULT 0,
    carbs_g          REAL        NOT NULL DEFAULT 0,
    fat_g            REAL        NOT NULL DEFAULT 0,
    description      TEXT        NOT NULL DEFAULT '',
    origin           TEXT        NOT NULL DEFAULT '',
    cooking_style    TEXT        NOT NULL DEFAULT '',
    prep_minutes     INTEGER     NOT NULL DEFAULT 0,
    cook_minutes     INTEGER     NOT NULL DEFAULT 0,
    servings         INTEGER     NOT NULL DEFAULT 1,
    ingredients      JSONB       NOT NULL DEFAULT '[]'::JSONB,
    steps            JSONB       NOT NULL DEFAULT '[]'::JSONB,
    tags             TEXT[]      NOT NULL DEFAULT '{}',
    image_url        TEXT        NOT NULL,
    image_source_url TEXT,
    image_author     TEXT,
    image_license    TEXT,
    featured         BOOLEAN     NOT NULL DEFAULT FALSE,
    -- Trọng số cho weighted random; 1.0 là mặc định.
    weight           REAL        NOT NULL DEFAULT 1.0 CHECK (weight > 0),
    -- Văn bản không dấu, chữ thường để tìm kiếm.
    search_text      TEXT        NOT NULL DEFAULT '',
    rating_sum       INTEGER     NOT NULL DEFAULT 0,
    rating_count     INTEGER     NOT NULL DEFAULT 0,
    favorite_count   INTEGER     NOT NULL DEFAULT 0,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX foods_cuisine_idx   ON foods (cuisine);
CREATE INDEX foods_style_idx     ON foods (style);
CREATE INDEX foods_dish_type_idx ON foods (dish_type);
CREATE INDEX foods_soup_type_idx ON foods (soup_type);
CREATE INDEX foods_meals_idx     ON foods USING GIN (meals);
CREATE INDEX foods_price_idx     ON foods (price_min, price_max);

CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email         TEXT        NOT NULL,
    phone         TEXT        NOT NULL,
    display_name  TEXT        NOT NULL,
    password_hash TEXT        NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login_at TIMESTAMPTZ
);

CREATE UNIQUE INDEX users_email_uq ON users (lower(email));
CREATE UNIQUE INDEX users_phone_uq ON users (phone);

CREATE TABLE food_ratings (
    user_id    UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    food_id    UUID        NOT NULL REFERENCES foods (id) ON DELETE CASCADE,
    score      SMALLINT    NOT NULL CHECK (score BETWEEN 1 AND 5),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, food_id)
);

CREATE TABLE favorite_foods (
    user_id    UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    food_id    UUID        NOT NULL REFERENCES foods (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, food_id)
);

CREATE INDEX favorite_foods_food_idx ON favorite_foods (food_id);

-- Quán ăn yêu thích. Dữ liệu quán đến từ nhà cung cấp địa điểm (Google / OpenStreetMap),
-- nên lưu kèm provider + place_ref để truy lại nguồn.
CREATE TABLE favorite_places (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID             NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    provider   TEXT             NOT NULL,
    place_ref  TEXT             NOT NULL,
    name       TEXT             NOT NULL,
    address    TEXT,
    lat        DOUBLE PRECISION NOT NULL,
    lng        DOUBLE PRECISION NOT NULL,
    cuisine    TEXT,
    food_id    UUID             REFERENCES foods (id) ON DELETE SET NULL,
    maps_url   TEXT,
    note       TEXT,
    created_at TIMESTAMPTZ      NOT NULL DEFAULT now(),
    UNIQUE (user_id, provider, place_ref)
);

CREATE INDEX favorite_places_user_idx ON favorite_places (user_id, created_at DESC);
