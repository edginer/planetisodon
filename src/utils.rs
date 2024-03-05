use std::collections::HashMap;

use chrono::NaiveDateTime;
use worker::{Date, Response};

pub fn shift_jis_url_encodeded_body_to_vec(
    data: &str,
) -> std::result::Result<HashMap<&str, String>, ()> {
    fn ascii_hex_digit_to_byte(value: u8) -> std::result::Result<u8, ()> {
        if value.is_ascii_hexdigit() {
            if value.is_ascii_digit() {
                // U+0030 '0' - U+0039 '9',
                Ok(value - 0x30)
            } else if value.is_ascii_uppercase() {
                // U+0041 'A' - U+0046 'F',
                Ok(value - 0x41 + 0xa)
            } else if value.is_ascii_lowercase() {
                // U+0061 'a' - U+0066 'f',
                Ok(value - 0x61 + 0xa)
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }

    data.split('&')
        .map(|x| {
            let split = x.split('=').collect::<Vec<_>>();
            if split.len() != 2 {
                return std::result::Result::Err(());
            }
            let (key, value) = (split[0], split[1]);
            let bytes = value.as_bytes();
            let len = bytes.len();
            let mut i = 0;
            let mut result = Vec::new();
            while i < len {
                let item = bytes[i];
                if item == 0x25 {
                    // Look up the next two bytes from 0x25
                    if let Some([next1, next2]) = bytes.get(i + 1..i + 3) {
                        let first_byte = ascii_hex_digit_to_byte(*next1)?;
                        let second_byte = ascii_hex_digit_to_byte(*next2)?;
                        let code = first_byte * 0x10_u8 + second_byte;
                        result.push(code);
                    }
                    i += 2;
                } else if item == 0x2b {
                    result.push(0x20);
                } else {
                    result.push(bytes[i]);
                }
                i += 1;
            }
            let result = encoding_rs::SHIFT_JIS.decode(&result).0.to_string();
            Ok((key, result))
        })
        .collect::<std::result::Result<HashMap<_, _>, ()>>()
}

pub fn response_shift_jis_text_plain(body: &str) -> worker::Result<Response> {
    let data = encoding_rs::SHIFT_JIS.encode(body).0.into_owned();
    let Ok(mut resp) = Response::from_bytes(data) else {
        return Response::error("internal server error - converting sjis", 500);
    };
    let _ = resp.headers_mut().delete("Content-Type");
    let _ = resp.headers_mut().append("Content-Type", "text/plain");
    Ok(resp)
}

pub fn response_shift_jis_text_plain_with_cache(
    body: &str,
    ttl: usize,
) -> worker::Result<Response> {
    let mut resp = response_shift_jis_text_plain(body)?;

    match ttl {
        1 => {
            let _ = resp.headers_mut().append("Cache-Control", "s-maxage=1");
        }
        3600 => {
            let _ = resp.headers_mut().append("Cache-Control", "s-maxage=3600");
        }
        86400 => {
            let _ = resp.headers_mut().append("Cache-Control", "s-maxage=86400");
        }
        s => {
            let max_age = format!("s-maxage={}", s);
            let _ = resp.headers_mut().append("Cache-Control", max_age.as_str());
        }
    }

    Ok(resp)
}

pub fn response_shift_jis_text_html(body: String) -> worker::Result<Response> {
    let data = encoding_rs::SHIFT_JIS.encode(&body).0.into_owned();
    let Ok(mut resp) = Response::from_bytes(data) else {
        return Response::error("internal server error - converting sjis", 500);
    };
    let _ = resp.headers_mut().delete("Content-Type");
    let _ = resp
        .headers_mut()
        .append("Content-Type", "text/html; charset=x-sjis");
    Ok(resp)
}

pub fn get_current_date_time() -> NaiveDateTime {
    let date = NaiveDateTime::from_timestamp_millis(Date::now().as_millis() as i64).unwrap();
    date.checked_add_signed(chrono::Duration::hours(9)).unwrap()
}

pub fn get_current_date_time_string(is_ja: bool) -> String {
    if is_ja {
        let dt = get_current_date_time();
        let dt_str = dt.format("%Y/%m/%d({weekday}) %H:%M:%S.%3f").to_string();
        let en_weekday = dt.format("%a").to_string();
        let weekday = convert_weekday_to_ja(&en_weekday);

        dt_str.replace("{weekday}", weekday)
    } else {
        get_current_date_time()
            .format("%Y/%m/%d(%a) %H:%M:%S.%3f")
            .to_string()
    }
}

pub fn convert_weekday_to_ja(weekday: &str) -> &str {
    match weekday {
        "Mon" => "月",
        "Tue" => "火",
        "Wed" => "水",
        "Thu" => "木",
        "Fri" => "金",
        "Sat" => "土",
        "Sun" => "日",
        x => x,
    }
}
