use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::CookieJar;

use super::{
    dto::{
        AuthResponse, ChangePasswordRequest, ForgotPasswordRequest, LoginRequest, MessageResponse, OtpSentResponse,
        RegisterOtpRequest, UpdateProfileRequest, UserProfile, VerifyOtpRequest,
    },
    session,
};
use crate::{
    error::{AppResult, ErrorBody},
    extract::{AuthUser, ClientIp, ValidatedJson},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/register/request-otp", post(request_register_otp))
        .route("/auth/register/verify", post(verify_register))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(me).patch(update_me))
        .route("/auth/password/forgot", post(forgot_password))
        .route("/auth/password/reset", post(reset_password))
        .route("/auth/password/change", post(change_password))
}

fn with_session(state: &AppState, jar: CookieJar, user: &UserProfile) -> AppResult<CookieJar> {
    let token = state.auth.issue_session(user.id)?;
    Ok(jar.add(session::cookie(token, state.cfg.session_ttl_hours, state.cfg.cookie_secure)))
}

/// Đăng ký — bước 1: gửi mã OTP về email
///
/// Nhận email và số điện thoại, gửi mã 6 chữ số (hiệu lực 10 phút) tới email.
#[utoipa::path(
    post, path = "/api/v1/auth/register/request-otp", tag = "Auth",
    request_body = RegisterOtpRequest,
    responses(
        (status = 202, body = OtpSentResponse),
        (status = 409, body = ErrorBody, description = "Email hoặc số điện thoại đã được đăng ký"),
        (status = 422, body = ErrorBody),
        (status = 429, body = ErrorBody),
    )
)]
pub async fn request_register_otp(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    ValidatedJson(body): ValidatedJson<RegisterOtpRequest>,
) -> AppResult<(StatusCode, Json<OtpSentResponse>)> {
    let sent = state.auth.request_register_otp(&body.email, &body.phone, &ip).await?;
    Ok((StatusCode::ACCEPTED, Json(sent)))
}

/// Đăng ký — bước 2: xác thực OTP, tạo tài khoản và cấp mật khẩu
///
/// OTP đúng thì tài khoản được tạo, hệ thống sinh mật khẩu và gửi về email, đồng thời đăng nhập luôn (đặt cookie phiên).
#[utoipa::path(
    post, path = "/api/v1/auth/register/verify", tag = "Auth",
    request_body = VerifyOtpRequest,
    responses((status = 201, body = AuthResponse), (status = 400, body = ErrorBody), (status = 422, body = ErrorBody))
)]
pub async fn verify_register(
    State(state): State<AppState>,
    jar: CookieJar,
    ValidatedJson(body): ValidatedJson<VerifyOtpRequest>,
) -> AppResult<(StatusCode, CookieJar, Json<AuthResponse>)> {
    let (user, emailed) = state.auth.verify_register(&body.email, &body.otp).await?;
    let jar = with_session(&state, jar, &user)?;
    let message = if emailed {
        "Tạo tài khoản thành công. Mật khẩu đăng nhập đã được gửi về email của bạn.".to_string()
    } else {
        "Tạo tài khoản thành công nhưng chưa gửi được email mật khẩu. Hãy dùng chức năng quên mật khẩu để nhận mật khẩu.".to_string()
    };
    Ok((StatusCode::CREATED, jar, Json(AuthResponse { user, message })))
}

/// Đăng nhập bằng email hoặc số điện thoại và mật khẩu
#[utoipa::path(
    post, path = "/api/v1/auth/login", tag = "Auth",
    request_body = LoginRequest,
    responses((status = 200, body = AuthResponse), (status = 401, body = ErrorBody), (status = 429, body = ErrorBody))
)]
pub async fn login(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    jar: CookieJar,
    ValidatedJson(body): ValidatedJson<LoginRequest>,
) -> AppResult<(CookieJar, Json<AuthResponse>)> {
    let user = state.auth.login(&body.identifier, &body.password, &ip).await?;
    let jar = with_session(&state, jar, &user)?;
    Ok((jar, Json(AuthResponse { user, message: "Đăng nhập thành công".into() })))
}

/// Đăng xuất
#[utoipa::path(post, path = "/api/v1/auth/logout", tag = "Auth", responses((status = 200, body = MessageResponse)))]
pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> (CookieJar, Json<MessageResponse>) {
    (jar.add(session::clear_cookie(state.cfg.cookie_secure)), Json(MessageResponse { message: "Đã đăng xuất".into() }))
}

/// Thông tin tài khoản đang đăng nhập
#[utoipa::path(
    get, path = "/api/v1/auth/me", tag = "Auth",
    responses((status = 200, body = UserProfile), (status = 401, body = ErrorBody)),
    security(("session_cookie" = []))
)]
pub async fn me(State(state): State<AppState>, AuthUser(user_id): AuthUser) -> AppResult<Json<UserProfile>> {
    Ok(Json(state.auth.profile(user_id).await?))
}

/// Cập nhật tên hiển thị
#[utoipa::path(
    patch, path = "/api/v1/auth/me", tag = "Auth",
    request_body = UpdateProfileRequest,
    responses((status = 200, body = UserProfile), (status = 401, body = ErrorBody)),
    security(("session_cookie" = []))
)]
pub async fn update_me(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    ValidatedJson(body): ValidatedJson<UpdateProfileRequest>,
) -> AppResult<Json<UserProfile>> {
    Ok(Json(state.auth.update_profile(user_id, &body.display_name).await?))
}

/// Quên mật khẩu — bước 1: gửi mã OTP về email
#[utoipa::path(
    post, path = "/api/v1/auth/password/forgot", tag = "Auth",
    request_body = ForgotPasswordRequest,
    responses((status = 202, body = OtpSentResponse), (status = 429, body = ErrorBody))
)]
pub async fn forgot_password(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    ValidatedJson(body): ValidatedJson<ForgotPasswordRequest>,
) -> AppResult<(StatusCode, Json<OtpSentResponse>)> {
    let sent = state.auth.request_reset_otp(&body.email, &ip).await?;
    Ok((StatusCode::ACCEPTED, Json(sent)))
}

/// Quên mật khẩu — bước 2: xác thực OTP, cấp mật khẩu mới qua email
#[utoipa::path(
    post, path = "/api/v1/auth/password/reset", tag = "Auth",
    request_body = VerifyOtpRequest,
    responses((status = 200, body = MessageResponse), (status = 400, body = ErrorBody), (status = 422, body = ErrorBody))
)]
pub async fn reset_password(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<VerifyOtpRequest>,
) -> AppResult<Json<MessageResponse>> {
    state.auth.verify_reset(&body.email, &body.otp).await?;
    Ok(Json(MessageResponse { message: "Mật khẩu mới đã được gửi về email của bạn.".into() }))
}

/// Đổi mật khẩu
#[utoipa::path(
    post, path = "/api/v1/auth/password/change", tag = "Auth",
    request_body = ChangePasswordRequest,
    responses((status = 200, body = MessageResponse), (status = 401, body = ErrorBody), (status = 422, body = ErrorBody)),
    security(("session_cookie" = []))
)]
pub async fn change_password(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    ValidatedJson(body): ValidatedJson<ChangePasswordRequest>,
) -> AppResult<Json<MessageResponse>> {
    state.auth.change_password(user_id, &body.current_password, &body.new_password).await?;
    Ok(Json(MessageResponse { message: "Đổi mật khẩu thành công".into() }))
}
