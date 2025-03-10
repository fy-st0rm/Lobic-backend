use crate::config::DEV;

use cookie::{Cookie, SameSite};
use time::Duration;

pub fn create(key: &str, value: &str, exp_in_sec: i64) -> String {
	Cookie::build((key, value))
		.http_only(!DEV)
		.same_site(if DEV {SameSite::Lax} else {SameSite::None})
		.secure(!DEV)
		.path("/")
		.max_age(Duration::new(exp_in_sec, 0))
		.build()
		.to_string()
}
