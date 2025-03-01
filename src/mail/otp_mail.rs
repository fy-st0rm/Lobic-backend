use lettre::message::SinglePart;
use lettre::Message;

pub fn otp_mail(to: &str, otp: String) -> Message {
	let smtp_username = std::env::var("SMTP_USERNAME").expect("'SMTP_USERNAME' must be set in .env file");

	Message::builder()
		.from(smtp_username.parse().unwrap())
		.to(to.parse().unwrap())
		.subject("OTP Verification")
		.singlepart(SinglePart::html(format!("<h1>{otp}</h1>")))
		.unwrap()
}
