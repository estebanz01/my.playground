mod short_url;
mod web;

use std::{env, io};

use qrcodegen::{QrCode, QrCodeEcc};
use short_url::ShortURL;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut run_web_server: bool = false;

    if args.len() > 1 {
        run_web_server = &args[1] == "--web";
    }

    if run_web_server {
        web::web();
    } else {
        console_mode();
    }
}

fn console_mode() {
    println!("Enter URL to short:");

    let mut url = String::new();

    io::stdin()
        .read_line(&mut url)
        .expect("Failed to read input");
    url = url.trim().to_string();

    let mut short_url: ShortURL;

    match ShortURL::new(&url, None) {
        Ok(value) => short_url = value,
        Err(e) => panic!("{}", e),
    }

    println!("Short URL is: {} with TTL: {}", short_url.id, short_url.ttl);

    if short_url.register() {
        println!("Key registered");
    } else {
        println!("Error trying to register key");
    }

    let qr = QrCode::encode_text(
        &format!("http://127.0.0.1/{}", short_url.id),
        QrCodeEcc::Medium,
    )
    .unwrap();
    println!("QR code");
    print_qr(&qr);

    std::thread::sleep(std::time::Duration::from_secs(5));

    short_url.expire();

    println!("new TTL: {}", short_url.ttl);
}

fn print_qr(qr: &QrCode) {
    let border: i32 = 4;
    for y in -border..qr.size() + border {
        for x in -border..qr.size() + border {
            let c: char = if qr.get_module(x, y) { '█' } else { ' ' };
            print!("{0}{0}", c);
        }
        println!();
    }
    println!();
}
