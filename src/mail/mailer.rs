use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

pub fn send_mail(email: Message) {
	let smtp_host = std::env::var("SMTP_HOST").expect("'SMTP_HOST' must be set in .env file");
	let smtp_username = std::env::var("SMTP_USERNAME").expect("'SMTP_USERNAME' must be set in .env file");
	let smtp_password = std::env::var("SMTP_PASSWORD").expect("'SMTP_PASSWORD' must be set in .env file");

	let creds = Credentials::new(smtp_username, smtp_password);
	let mailer = SmtpTransport::starttls_relay(&smtp_host)
		.unwrap()
		.credentials(creds)
		.build();

	mailer.send(&email).unwrap();
}
