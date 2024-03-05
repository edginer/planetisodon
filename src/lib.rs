use cookie::Cookie;
use dtos::Board;
use planetscale_driver::PSConnection;
use routes::{
    auth::route_auth, bbs_cgi::route_bbs_cgi, dat_routing::route_dat,
    setting_txt::route_setting_txt, subject_txt::route_subject_txt,
};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, OnceLock,
    },
};
use utils::response_shift_jis_text_plain_with_cache;
use worker::*;

use bbs_repository::BbsRepository;

mod utils;
mod routes {
    pub(crate) mod auth;
    pub(crate) mod bbs_cgi;
    pub(crate) mod dat_routing;
    pub(crate) mod setting_txt;
    pub(crate) mod subject_txt;
}
mod bbs_repository;
mod dtos;

fn get_connection(host: &str, user: &str, password: &str) -> PSConnection {
    static PLANETSCALE_CONN: OnceLock<PSConnection> = OnceLock::new();

    PLANETSCALE_CONN
        .get_or_init(|| PSConnection::new(host, user, password))
        .to_owned()
}

fn get_user_token_cookie(req: &Request) -> Option<String> {
    let cookie_str = req.headers().get("Cookie").ok()??;
    for cookie in Cookie::split_parse(cookie_str).flatten() {
        if cookie.name() == "user_token" {
            return Some(cookie.value().to_string());
        }
    }
    None
}

struct BoardsCtx {
    boards: Vec<Board>,
    board_key_to_board: HashMap<String, usize>,
    board_id_to_board: HashMap<i32, usize>,
}

impl BoardsCtx {
    fn new(boards: Vec<Board>) -> Self {
        let mut board_key_to_board = HashMap::new();
        let mut board_id_to_board = HashMap::new();
        for (idx, board) in boards.iter().enumerate() {
            board_key_to_board.insert(board.board_key.clone(), idx);
            board_id_to_board.insert(board.id, idx);
        }
        Self {
            boards,
            board_key_to_board,
            board_id_to_board,
        }
    }

    fn get_board_by_key(&self, key: &str) -> Option<&Board> {
        let idx = self.board_key_to_board.get(key)?;
        self.boards.get(*idx)
    }

    fn get_board_by_id(&self, id: i32) -> Option<&Board> {
        let idx = self.board_id_to_board.get(&id)?;
        self.boards.get(*idx)
    }
}

async fn get_boards(repo: BbsRepository) -> Arc<BoardsCtx> {
    static BOARDS_CTX_CACHE: OnceLock<Arc<BoardsCtx>> = OnceLock::new();
    static LAST_MODIFIED: AtomicU64 = AtomicU64::new(0); // unix timestamp (seconds)
    const BOARDS_LIST_CACHE_TTL: u64 = 60 * 5;
    let y = Date::now().as_millis() / 1000;
    let x = LAST_MODIFIED.load(Ordering::Relaxed);

    if x != 0 && y - x <= BOARDS_LIST_CACHE_TTL && BOARDS_CTX_CACHE.get().is_some() {
        return BOARDS_CTX_CACHE.get().unwrap().to_owned();
    }

    let boards = repo.get_boards().await.unwrap();

    let boards = Arc::new(BoardsCtx::new(boards));
    if BOARDS_CTX_CACHE.set(boards.clone()).is_ok() {
        LAST_MODIFIED.store(y, Ordering::Relaxed);
    }
    boards
}

#[derive(Debug, Clone)]
struct GoogleOAuth2 {
    client_id: String,
    client_secret: String,
}

struct Ctx {
    db_conn: PSConnection,
    google_oauth2: GoogleOAuth2,
    bbs_repository: BbsRepository,
    boards: Arc<BoardsCtx>,
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let (host, username, password) = (
        env.var("DATABASE_HOST")?.to_string(),
        env.var("DATABASE_USERNAME")?.to_string(),
        env.var("DATABASE_PASSWORD")?.to_string(),
    );

    let db_conn = get_connection(&host, &username, &password);
    let repo = BbsRepository::new(db_conn.clone());

    worker::Router::with_data(Ctx {
        bbs_repository: repo.clone(),
        db_conn,
        google_oauth2: GoogleOAuth2 {
            client_id: env.secret("GOOGLE_CLIENT_ID")?.to_string(),
            client_secret: env.secret("GOOGLE_CLIENT_SECRET")?.to_string(),
        },
        boards: get_boards(repo).await,
    })
    .get("/", |_, _| {
        let html = include_str!("static/index.html");
        Response::from_html(html)
    })
    .get_async("/auth", route_auth)
    .post_async("/test/bbs.cgi", route_bbs_cgi)
    .get_async("/:boardKey/subject.txt", route_subject_txt)
    .get_async("/:boardKey/SETTING.TXT", route_setting_txt)
    .get_async("/:boardKey/dat/:threadKey", route_dat)
    .get("/:boardKey/head.txt", |_, _| {
        response_shift_jis_text_plain_with_cache("<a href=\"/\">こちらへ</a>", 3600)
    })
    .run(req, env)
    .await
}
