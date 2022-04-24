use data_encoding::BASE64URL_NOPAD;
use rand::Rng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use regex::Regex;

#[readonly::make]
pub struct ShortURL {
    pub id: String,
    pub url: String, // Reference to the "real" URL.
    pub created_at: i64,
    pub ttl: u16, // TTL for the short url.
}

// Function to calculate the short slug of the specified URL.
fn calculate_short(url: &str) -> String {
    let mut rng: Pcg64 = Seeder::from(&url.to_lowercase()).make_rng();

    BASE64URL_NOPAD.encode(&rng.gen::<u16>().to_ne_bytes())
}

// Validates via regex if the specified URL is valid.
fn is_valid_url(url: &str) -> bool {
    let re: Regex =
        Regex::new(r"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,30}\b([-a-zA-Z0-9()!@:%_\+.~#?&//=]*)").unwrap();

    re.is_match(&url.to_lowercase())
}

impl ShortURL {
    pub fn new(url: &str, ttl: Option<u16>) -> Result<Self, String> {
        if is_valid_url(url) {
            Ok(Self {
                id: calculate_short(url),
                url: (&url).to_string(), // I just found out we need to respect capitalisation
                ttl: ttl.unwrap_or(3600), // If None, default to 1h (in secs).
                created_at: chrono::Utc::now().timestamp(),
            })
        } else {
            Err("URL Provided is not valid.".to_string())
        }
    }

    pub fn register(&self) -> bool {
        match simple_redis::create("redis://127.0.0.1:6379/") {
            Ok(mut client) => {
                match client.setex(self.id.as_str(), self.url.as_str(), self.ttl.into()) {
                    Ok(_) => return true,
                    Err(error) => {
                        println!("Error registering the URL: {}", error);
                        return false
                    }
                };
            }
            Err(error) => {
                println!("Unable to connect to redis: {}", error);
                return false;
            }
        };
    }

    pub fn expire(&mut self) -> bool {
        match simple_redis::create("redis://127.0.0.1:6379/") {
            Ok(mut client) => {
                match client.expire(&self.id, 0) {
                    Ok(_) => {
                        self.ttl = 0;
                        return true;
                    }
                    Err(error) => {
                        println!("Cannot expire key: {}", error);
                        return false;
                    }
                };
            }
            Err(error) => {
                println!("Unable to connect to redis: {}", error);
                return false;
            }
        };
    }
}
