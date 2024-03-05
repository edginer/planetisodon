use cookie::Cookie;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RevocationUrl, Scope, TokenResponse,
    TokenUrl,
};
use planetscale_driver::query;
use serde::Deserialize;
use sha3::Digest;
use uuid::{NoContext, Timestamp};
use worker::{Date, Request, Response, Result, RouteContext};

use crate::{dtos::User, get_user_token_cookie, utils::response_shift_jis_text_html, Ctx};

#[derive(Debug, Clone, Deserialize)]
struct GoogleUserInfo {
    sub: String,
    #[allow(dead_code)]
    email: String,
    email_verified: bool,
    #[allow(dead_code)]
    picture: String,
}

fn get_pkce_checker_cookie(req: &Request) -> Option<String> {
    let cookie_str = req.headers().get("Cookie").ok()??;
    for cookie in Cookie::split_parse(cookie_str).flatten() {
        if cookie.name() == "pkce_checker" {
            return Some(cookie.value().to_string());
        }
    }
    None
}

pub async fn route_auth(req: Request, ctx: RouteContext<Ctx>) -> Result<Response> {
    let cookie = get_user_token_cookie(&req);
    if let Some(cookie) = cookie {
        let conn = ctx.data.db_conn.clone();
        let user = query("SELECT * FROM users WHERE user_hash = '$0' LIMIT 1")
            .bind(&cookie)
            .fetch_one::<User>(&conn)
            .await;
        if user.is_ok() {
            return Response::ok(format!(
                "Your account is already logged in. \nToken: #{cookie}"
            ));
        }
    }

    let url = req.url().unwrap();
    let queries = url.query_pairs();
    let code = queries
        .clone()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.to_string());
    let state = queries
        .clone()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string());
    let pkce_checker = get_pkce_checker_cookie(&req);
    // Logging process using Google OAuth 2.0
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .expect("Invalid token endpoint URL");
    // Set up the config for the Google OAuth2 process.
    let client = BasicClient::new(
        ClientId::new(ctx.data.google_oauth2.client_id),
        Some(ClientSecret::new(ctx.data.google_oauth2.client_secret)),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(
        RedirectUrl::new(ctx.env.var("GOOGLE_AUTH_REDIRECT_URI").unwrap().to_string())
            .expect("Invalid redirect URL"),
    )
    .set_revocation_uri(
        RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
            .expect("Invalid revocation endpoint URL"),
    );

    if code.is_none() && state.is_none() {
        // Google supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
        // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
        let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

        let pkce_checker_uuid = uuid::Uuid::new_v4();

        ctx.env
            .kv("planetisodongoogle_pkce_code_verifier")
            .unwrap()
            .put(&pkce_checker_uuid.to_string(), pkce_code_verifier.secret())
            .unwrap()
            .expiration_ttl(300)
            .execute()
            .await
            .unwrap();

        // Generate the authorization URL to which we'll redirect the user.
        let (authorize_url, csrf_state) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/userinfo.email".to_string(),
            ))
            .set_pkce_challenge(pkce_code_challenge)
            .url();

        ctx.env
            .kv("planetisodon_google_csrf_state")
            .unwrap()
            .put(&pkce_checker_uuid.to_string(), csrf_state.secret())
            .unwrap()
            .expiration_ttl(300)
            .execute()
            .await
            .unwrap();

        response_shift_jis_text_html(format!(
            "トークンを取得するためにGoogleにログインしてください <br>
                <a href=\"{authorize_url}\">Login with Google</a>",
        ))
        .map(|mut x| {
            x.headers_mut()
                .append("Set-Cookie", &format!("pkce_checker={pkce_checker_uuid}"))
                .unwrap();
            x
        })
    } else if code.is_some() && state.is_some() && pkce_checker.is_some() {
        let pkce_code_verifier = ctx
            .env
            .kv("planetisodongoogle_pkce_code_verifier")
            .unwrap()
            .get(pkce_checker.as_ref().unwrap())
            .text()
            .await
            .unwrap()
            .unwrap();
        let csrf_state = ctx
            .env
            .kv("planetisodon_google_csrf_state")
            .unwrap()
            .get(&pkce_checker.unwrap())
            .text()
            .await
            .unwrap()
            .unwrap();

        if csrf_state != state.unwrap() {
            return Response::error("Unauthorized - CSRF state mismatch", 401);
        }

        let token_response = client
            .exchange_code(AuthorizationCode::new(code.unwrap()))
            .set_pkce_verifier(PkceCodeVerifier::new(pkce_code_verifier))
            .request_async(oauth2::reqwest::async_http_client)
            .await;

        let token_response = token_response.unwrap();

        // Get UserInfo from Google API
        let user_info = reqwest::Client::new()
            .get(format!(
                "https://www.googleapis.com/oauth2/v3/userinfo?access_token={}",
                token_response.access_token().secret()
            ))
            .send()
            .await
            .unwrap()
            .json::<GoogleUserInfo>()
            .await
            .unwrap();

        if !user_info.email_verified {
            return Response::error(
                "Unauthorized - your google account is not email verified",
                401,
            );
        }

        let sub_hash = {
            let salt = ctx.env.secret("USER_SUB_HASH_SALT").unwrap().to_string();
            let mut sub_hash = user_info.sub;
            for _ in 0..10 {
                let hash = sha3::Sha3_256::digest(format!("{sub_hash}{salt}").as_bytes());
                sub_hash = hash.iter().fold(String::new(), |mut acc, x| {
                    acc.push_str(&format!("{:02x}", x));
                    acc
                });
            }
            sub_hash.truncate(24);
            sub_hash
        };

        let conn = ctx.data.db_conn.clone();

        let ip_addr = req.headers().get("cf-connecting-ip").unwrap().unwrap();
        let user_id = uuid::Uuid::new_v7(Timestamp::from_unix(
            NoContext,
            Date::now().as_millis() / 1000,
            ((Date::now().as_millis() % 1000) * 1000) as u32,
        ));

        if query("SELECT * FROM users WHERE user_hash = '$0' LIMIT 1")
            .bind(&sub_hash)
            .fetch_one::<User>(&conn)
            .await
            .is_err()
        {
            query("INSERT INTO users (user_hash, ip_address, id) VALUES ('$0', '$1', '$2')")
                .bind(&sub_hash)
                .bind(ip_addr)
                .bind(user_id.to_string())
                .execute(&conn)
                .await
                .unwrap();
        };

        Response::ok(format!("token: #{sub_hash}")).map(|mut x| {
            x.headers_mut()
                .append("Set-Cookie", &format!("user_token={sub_hash}"))
                .unwrap();
            x
        })
    } else {
        Response::error("Bad Request", 400)
    }
}
