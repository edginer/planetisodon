use base64::{engine::general_purpose, Engine};
use pwhash::unix;
use regex::Regex;
use sha1::{Digest, Sha1};
use worker::{Request, Response, Result, RouteContext};

use crate::{
    bbs_repository::{CreatingResponse, CreatingThread},
    get_user_token_cookie,
    utils::{
        self, get_current_date_time, get_current_date_time_string, response_shift_jis_text_html,
    },
    Ctx,
};

fn sanitize(input: &str) -> String {
    input
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\n', "<br>")
        .replace('\r', "")
        .replace("&#10;", "")
}

fn sanitize_thread_name(input: &str) -> String {
    let sanitized = sanitize(input);
    // Delete all of semicolon closing \n character references
    let re = Regex::new(r"&#([Xx]0*[aA]|0*10);").unwrap();
    let rn_sanitized = re.replace_all(&sanitized, "");

    sanitize_non_semi_closing_num_char_refs(&rn_sanitized)
}

// Delete all of non-semicolon closing numeric character references
fn sanitize_non_semi_closing_num_char_refs(target: &str) -> String {
    let mut sanitized = Vec::new();
    let mut ampersand_used = -1;
    let mut total_removed_len = 0;
    enum NumRefKind {
        Undef, // this state is only cause after reading "&#"
        Hex,
        Dec,
    }
    let mut in_num_ref = None;
    for (i, c) in target.chars().enumerate() {
        if let Some(kind) = &in_num_ref {
            if c == ';' {
                in_num_ref = None;
                sanitized.push(c);
            } else {
                match kind {
                    NumRefKind::Undef => {
                        match c {
                            'x' | 'X' => in_num_ref = Some(NumRefKind::Hex),
                            '0'..='9' => in_num_ref = Some(NumRefKind::Dec),
                            _ => in_num_ref = None,
                        };
                        sanitized.push(c);
                    }
                    NumRefKind::Hex => match c {
                        '0'..='9' | 'a'..='f' | 'A'..='F' => sanitized.push(c),
                        _ => {
                            // invalid non-semicolon closing numeric character references
                            in_num_ref = None;
                            sanitized =
                                sanitized[0..ampersand_used as usize - total_removed_len].to_vec();
                            total_removed_len += i - ampersand_used as usize;
                            sanitized.push(c);
                            if c == '&' {
                                ampersand_used = i as isize;
                            }
                        }
                    },
                    NumRefKind::Dec => match c {
                        '0'..='9' => sanitized.push(c),
                        _ => {
                            // invalid non-semicolon closing numeric character references
                            in_num_ref = None;
                            sanitized =
                                sanitized[0..ampersand_used as usize - total_removed_len].to_vec();
                            total_removed_len += i - ampersand_used as usize;
                            sanitized.push(c);
                            if c == '&' {
                                ampersand_used = i as isize;
                            }
                        }
                    },
                }
            }
        } else {
            sanitized.push(c);
            if c == '&' {
                ampersand_used = i as isize;
            } else if ampersand_used == (i as isize - 1) && c == '#' {
                in_num_ref = Some(NumRefKind::Undef);
            }
        }
    }

    if in_num_ref.is_some() {
        sanitized = sanitized[0..ampersand_used as usize - total_removed_len].to_vec();
    }

    sanitized.into_iter().collect::<String>()
}

#[derive(Debug, Clone)]
struct BbsCgiForm {
    subject: Option<String>,
    name: String,
    mail: String,
    body: String,
    board_key: String,
    is_thread: bool,
    thread_id: Option<String>,
    cap: Option<String>,
}

pub struct TokenRemover {
    regex: Regex,
}

impl TokenRemover {
    pub(crate) fn new() -> TokenRemover {
        TokenRemover {
            regex: Regex::new(r"[a-z0-9]{30,}?").unwrap(),
        }
    }

    pub(crate) fn remove(&self, name: String) -> String {
        if name.len() >= 20 && self.regex.is_match(&name) {
            String::new()
        } else {
            name
        }
    }
}

fn calculate_author_id(ip_addr: &str) -> String {
    let date = get_current_date_time().date().to_string();
    let mut id = calculate_trip(&format!("{ip_addr}{date}"));
    id.truncate(9);
    id
}

fn extract_forms(bytes: Vec<u8>) -> Option<BbsCgiForm> {
    let data = encoding_rs::SHIFT_JIS.decode(&bytes).0.to_string();

    let Ok(result) = utils::shift_jis_url_encodeded_body_to_vec(&data) else {
        return None;
    };
    let is_thread = {
        let submit = &result["submit"];
        match submit as &str {
            "書き込む" => false,
            "新規スレッド作成" => true,
            // TODO: above comment
            _ => return None,
        }
    };

    let mail_segments = result["mail"].split('#').collect::<Vec<_>>();
    let mail = mail_segments[0];
    let cap = if mail_segments.len() == 1 {
        None
    } else {
        Some(sanitize(&mail_segments[1..].concat()))
    };

    let subject = if is_thread {
        Some(sanitize_thread_name(&result["subject"]).clone())
    } else {
        None
    };

    let name_segments = result["FROM"].split('#').collect::<Vec<_>>();
    let name = name_segments[0];
    let name = if name_segments.len() == 1 {
        let token_remover = TokenRemover::new();
        let name = token_remover.remove(name.to_string());
        sanitize(&name)
            .replace('◆', "◇")
            .replace("&#9670;", "◇")
            .replace('★', "☆")
            .replace("&#9733;", "☆")
    } else {
        // TODO: smell
        let trip = sanitize(&name_segments[1..].concat())
            .replace('◆', "◇")
            .replace("&#9670;", "◇");
        let trip = calculate_trip(&trip);
        format!("{name}◆{trip}")
    };

    let mail = sanitize(mail).to_string();
    let body = sanitize(&result["MESSAGE"]).clone();
    let board_key = result["bbs"].clone();

    let thread_id = if is_thread {
        None
    } else {
        Some(result["key"].clone())
    };

    Some(BbsCgiForm {
        subject,
        name,
        mail,
        body,
        board_key,
        is_thread,
        thread_id,
        cap,
    })
}

// &str is utf-8 bytes
pub fn calculate_trip(target: &str) -> String {
    let bytes = encoding_rs::SHIFT_JIS.encode(target).0.into_owned();

    if bytes.len() >= 12 {
        let mut hasher = Sha1::new();
        hasher.update(&bytes);

        let calc_bytes = Vec::from(hasher.finalize().as_slice());
        let result = &general_purpose::STANDARD.encode(calc_bytes)[0..12];
        result.to_string().replace('+', ".")
    } else {
        let mut salt = Vec::from(if bytes.len() >= 3 { &bytes[1..=2] } else { &[] });
        salt.push(0x48);
        salt.push(0x2e);
        let salt = salt
            .into_iter()
            .map(|x| match x {
                0x3a..=0x40 => x + 7,
                0x5b..=0x60 => x + 6,
                46..=122 => x,
                _ => 0x2e,
            })
            .collect::<Vec<_>>();

        let salt = std::str::from_utf8(&salt).unwrap();
        let result = unix::crypt(bytes.as_slice(), salt).unwrap();
        result[3..].to_string()
    }
}

pub async fn route_bbs_cgi(mut req: Request, ctx: RouteContext<Ctx>) -> Result<Response> {
    let Ok(Some(ip_addr)) = req.headers().get("CF-Connecting-IP") else {
        return Response::error("internal server error - cf-connecting-ip", 500);
    };
    let Ok(req_bytes) = req.bytes().await else {
        return Response::error("Bad request - read bytes", 400);
    };
    let form = match extract_forms(req_bytes) {
        Some(form) => form,
        None => return Response::error("Bad request - extract forms", 400),
    };

    let Some(board) = ctx.data.boards.get_board_by_key(&form.board_key) else {
        return Response::error("Not Found - board not found", 404);
    };

    let (user_token, cookie_token) = match (get_user_token_cookie(&req), form.cap) {
        (Some(user_token), _) => (Some(user_token), true),
        (_, Some(cap)) => (Some(cap), false),
        _ => (None, false),
    };

    let user_token = if let Some(user_token) = &user_token {
        let user = ctx.data.bbs_repository.get_user(user_token).await.unwrap();
        if user.is_none() {
            return Response::error("Forbidden - given user token is invalid", 403).map(|mut x| {
                x.headers_mut()
                    .append("Set-Cookie", "user_token=; Max-Age=0; Path=/")
                    .unwrap();
                x
            });
        }
        if matches!(user, Some(user) if user.disabled == 1) {
            return Response::error("Forbidden - given user token is disabled", 403).map(
                |mut x| {
                    x.headers_mut()
                        .append("Set-Cookie", "user_token=; Max-Age=0; Path=/")
                        .unwrap();
                    x
                },
            );
        }
        user_token
    } else {
        let host_url = ctx.env.var("GOOGLE_AUTH_REDIRECT_URI").unwrap().to_string();
        return response_shift_jis_text_html(format!(
            r#"<html><!-- 2ch_X:error -->

<head>ＥＲＲＯＲ</head>

<body>
    以下にアクセスして認証してから書き込んでください<br>
    {host_url}

    認証後取得した#から始まるトークンをメール欄に入力し、書き込みを行ってください
</body>

</html>"#
        ));
    };

    if form.is_thread {
        let title = form.subject.unwrap();

        if let Err(e) = ctx
            .data
            .bbs_repository
            .create_thread(CreatingThread {
                board_id: board.id,
                title,
                name: form.name,
                mail: form.mail,
                body: form.body,
                date: get_current_date_time_string(true),
                author_id: calculate_author_id(&ip_addr),
                ip_addr,
                user_hash: user_token.clone(),
            })
            .await
        {
            return if e.to_string().contains("No results found") {
                Response::error("Not Found - board not found", 404)
            } else {
                Response::error("internal server error - create thread", 500)
            };
        }
    } else if let Err(e) = ctx
        .data
        .bbs_repository
        .create_response(CreatingResponse {
            board_id: board.id,
            thread_key: form.thread_id.unwrap().parse().unwrap(),
            name: form.name,
            mail: form.mail,
            body: form.body,
            date: get_current_date_time_string(true),
            author_id: calculate_author_id(&ip_addr),
            ip_addr,
            user_hash: user_token.clone(),
        })
        .await
    {
        return if e.to_string().contains("No results found") {
            Response::error("Not Found - thread not found", 404)
        } else {
            Response::error("internal server error - create response", 500)
        };
    }
    let data = encoding_rs::SHIFT_JIS
        .encode(
            r#"<html><!-- 2ch_X:true -->

    <head>
        <meta http-equiv="Content-Type" content="text/html; charset=x-sjis">
        <title>書きこみました</title>
    </head>
    
    <body>書きこみました</body>
    
    </html>"#,
        )
        .0
        .into_owned();
    let Ok(mut resp) = Response::from_bytes(data) else {
        return Response::error("internal server error - converting sjis", 500);
    };
    let _ = resp.headers_mut().delete("Content-Type");
    let _ = resp.headers_mut().append("Content-Type", "text/plain");
    if !cookie_token {
        let _ = resp.headers_mut().append(
            "Set-Cookie",
            &format!("user_token={user_token}; Max-Age=31536000; Path=/"),
        );
    }

    Ok(resp)
}
