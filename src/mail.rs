use lettre::message::header::ContentType;
use lettre::{Message, SmtpTransport, Transport};
use lettre::transport::smtp::authentication::Credentials;

pub fn send_mail(subject: &str, body: &str, sender: &str, receiver: &str, creds: Credentials) -> Result<(), String> {
    let email = Message::builder()
        .from(format!("<{}>", sender).parse().unwrap())
        .to(format!("<{}>", receiver).parse().unwrap())
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(String::from(body))
        .unwrap();

    let mailer = SmtpTransport::builder_dangerous("smtp.wondersgroup.com")
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => Ok(()),
        Err(e) => {
            Err(format!("Error sending email: {}", e))
        }
    }
}

pub fn test(username: &str, password: &str) -> bool {
    let mailer = SmtpTransport::builder_dangerous("smtp.wondersgroup.com")
        .credentials(Credentials::new(username.to_owned(), password.to_owned()))
        .build();
    mailer.test_connection().unwrap_or_else(|e| {
        eprintln!("Error connecting to SMTP server: {}", e);
        false
    })
}