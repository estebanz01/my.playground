use std::{collections::HashMap, fs};

use crate::ShortURL;
use qrcodegen::{QrCode, QrCodeEcc};
use warp::{http::Response, Filter};

fn a_response(
    status: u16,
    content_type: &str,
    body: &str,
) -> Result<Response<String>, warp::http::Error> {
    warp::http::Response::builder()
        .header("content-type", content_type.to_string())
        .status(status)
        .body(body.to_string())
}

fn a_redirect(status: u16, location: &str) -> Result<Response<String>, warp::http::Error> {
    let final_status: u16;

    match status {
        301 | 302 | 303 | 307 | 308 => final_status = status,
        _ => final_status = 302,
    }

    warp::http::Response::builder()
        .header("Location", location)
        .status(final_status)
        .body("".to_string())
}

#[tokio::main]
pub async fn web() {
    let root = warp::path::end().map(|| match fs::read_to_string("src/html/web.html") {
        Ok(body) => a_response(200, "text/html; charset=utf-8", &body),
        Err(_) => a_redirect(303, "/500"),
    });

    let to_404 = warp::path("404").map(|| match fs::read_to_string("src/html/404.html") {
        Ok(body) => a_response(404, "text/html; charset=utf-8", &body),
        Err(_) => a_response(404, "text/html; charset=utf-8", "Not Found"),
    });

    let to_500 = warp::path("500").map(|| match fs::read_to_string("src/html/500.html") {
        Ok(body) => a_response(500, "text/html; charset=utf-8", &body),
        Err(_) => a_response(500, "text/html; charset=utf-8", "Server Error"),
    });

    let new_url = warp::post()
        .and(warp::path("new"))
        .and(warp::body::form())
        .map(|original_url: HashMap<String, String>| {
            let short_url: ShortURL;

            for val in original_url.values() {
                println!("Param: {}", val);
            }

            match ShortURL::new(original_url.get("original").unwrap(), None) {
                Ok(value) => short_url = value,
                Err(e) => return a_response(500, "text/plain; charset=utf-8", &e.to_string()),
            }

            if short_url.register() {
                let qr = QrCode::encode_text(
                    &format!("http://127.0.0.1:3030/r/{}", short_url.id),
                    QrCodeEcc::Medium,
                )
                .unwrap();
                let svg = to_svg_string(&qr, 4);
                let html = fs::read_to_string("src/html/file.html").unwrap();

                let qr_code = html.replace("{}", &short_url.id).replace("<x-svg/>", &svg);

                a_response(200, "text/html; charset=utf-8", &qr_code)
            } else {
                a_response(
                    500,
                    "text/plain; charset=utf-8",
                    "Key couldn't be generated properly.",
                )
            }
        });

    let redirect = warp::path("r").and(warp::path::param()).map(|id: String| {
        match simple_redis::create("redis://127.0.0.1:6379/") {
            Ok(mut client) => match client.get_string(&id) {
                Ok(value) => a_redirect(302, &value),
                Err(_) => a_redirect(303, "/404"),
            },
            Err(_) => a_redirect(303, "/500"),
        }
    });

    let routes = warp::get()
        .and(root.or(to_404).or(to_500).or(redirect))
        .or(new_url);

    println!("Trying to run webserver in port 3030");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await
}

// Utility function to generate an SVG string QR code. Extracted from https://github.com/nayuki/QR-Code-generator/blob/master/rust-no-heap/examples/qrcodegen-demo.rs
fn to_svg_string(qr: &QrCode, border: i32) -> String {
    assert!(border >= 0, "Border must be non-negative");
    let mut result = String::new();
    result += "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";
    result += "<!DOCTYPE svg PUBLIC \"-//W3C//DTD SVG 1.1//EN\" \"http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd\">\n";
    let dimension = qr
        .size()
        .checked_add(border.checked_mul(2).unwrap())
        .unwrap();
    result += &format!(
		"<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" viewBox=\"0 0 {0} {0}\" stroke=\"none\">\n", dimension);
    result += "\t<rect width=\"40\" height=\"40\" fill=\"#FFFFFF\"/>\n";
    result += "\t<path d=\"";
    for y in 0..qr.size() {
        for x in 0..qr.size() {
            if qr.get_module(x, y) {
                if x != 0 || y != 0 {
                    result += " ";
                }
                result += &format!("M{},{}h1v1h-1z", x + border, y + border);
            }
        }
    }
    result += "\" fill=\"#000000\"/>\n";
    result += "</svg>\n";
    result
}
