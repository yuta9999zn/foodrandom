use anyhow::{Context, Result};
use lettre::{
    message::{header::ContentType, Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::config::{Config, SmtpTls};

/// Gửi email qua SMTP. Development trỏ tới Mailpit (xem thư ở http://localhost:8125),
/// production trỏ tới máy chủ SMTP thật — cùng một đường mã, không có nhánh giả lập.
#[derive(Clone)]
pub struct Mailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}

impl Mailer {
    pub fn new(cfg: &Config) -> Result<Self> {
        let mut builder = match cfg.smtp_tls {
            SmtpTls::None => AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&cfg.smtp_host),
            SmtpTls::StartTls => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.smtp_host)
                .context("cấu hình SMTP STARTTLS không hợp lệ")?,
            SmtpTls::Tls => {
                AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.smtp_host).context("cấu hình SMTP TLS không hợp lệ")?
            }
        }
        .port(cfg.smtp_port)
        .timeout(Some(std::time::Duration::from_secs(10)));

        if let (Some(user), Some(pass)) = (&cfg.smtp_username, &cfg.smtp_password) {
            builder = builder.credentials(Credentials::new(user.clone(), pass.clone()));
        }

        let from: Mailbox = cfg.mail_from.parse().context("MAIL_FROM không hợp lệ")?;
        Ok(Self { transport: builder.build(), from })
    }

    pub async fn send(&self, to: &str, subject: &str, text: String, html: String) -> Result<()> {
        let to: Mailbox = to.parse().context("địa chỉ email người nhận không hợp lệ")?;
        let message = Message::builder()
            .from(self.from.clone())
            .to(to)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(SinglePart::builder().header(ContentType::TEXT_PLAIN).body(text))
                    .singlepart(SinglePart::builder().header(ContentType::TEXT_HTML).body(html)),
            )
            .context("không dựng được email")?;
        self.transport.send(message).await.context("gửi email thất bại")?;
        Ok(())
    }

    pub async fn send_otp(&self, to: &str, code: &str, purpose_label: &str, ttl_minutes: u64) -> Result<()> {
        let subject = format!("[FoodRandom] Mã xác thực {purpose_label}: {code}");
        let text = format!(
            "Xin chào,\n\nMã xác thực {purpose_label} của bạn là: {code}\nMã có hiệu lực trong {ttl_minutes} phút. \
             Không chia sẻ mã này cho bất kỳ ai.\n\nNếu bạn không yêu cầu, hãy bỏ qua email này.\n\n— FoodRandom"
        );
        let html = layout(&format!(
            "<p>Xin chào,</p><p>Mã xác thực {purpose_label} của bạn là:</p>\
             <p style=\"font-size:32px;font-weight:700;letter-spacing:8px;color:#E85D78;margin:16px 0\">{code}</p>\
             <p>Mã có hiệu lực trong <b>{ttl_minutes} phút</b>. Không chia sẻ mã này cho bất kỳ ai.</p>\
             <p style=\"color:#8a7a7d\">Nếu bạn không yêu cầu, hãy bỏ qua email này.</p>"
        ));
        self.send(to, &subject, text, html).await
    }

    pub async fn send_password(&self, to: &str, password: &str, reason: &str) -> Result<()> {
        let subject = "[FoodRandom] Mật khẩu đăng nhập của bạn".to_string();
        let text = format!(
            "Xin chào,\n\n{reason}\nMật khẩu đăng nhập của bạn là: {password}\n\n\
             Hãy đăng nhập bằng email hoặc số điện thoại cùng mật khẩu này, sau đó đổi mật khẩu trong trang Tài khoản.\n\n— FoodRandom"
        );
        let html = layout(&format!(
            "<p>Xin chào,</p><p>{reason}</p><p>Mật khẩu đăng nhập của bạn là:</p>\
             <p style=\"font-size:22px;font-weight:700;font-family:monospace;background:#FFF3EF;border:1px solid #EAD5D1;\
             border-radius:12px;padding:12px 16px;display:inline-block;color:#30282A\">{password}</p>\
             <p>Hãy đăng nhập bằng email hoặc số điện thoại cùng mật khẩu này, sau đó đổi mật khẩu trong trang Tài khoản.</p>"
        ));
        self.send(to, &subject, text, html).await
    }
}

fn layout(inner: &str) -> String {
    format!(
        "<div style=\"font-family:'Be Vietnam Pro',Arial,sans-serif;background:#FFF3EF;padding:24px\">\
         <div style=\"max-width:480px;margin:0 auto;background:#fff;border:1px solid #EAD5D1;border-radius:16px;padding:28px;color:#30282A\">\
         <h2 style=\"margin:0 0 12px;color:#E85D78\">FoodRandom</h2>{inner}\
         <p style=\"margin-top:24px;color:#8a7a7d;font-size:12px\">Hôm nay ăn gì? — FoodRandom</p></div></div>"
    )
}
