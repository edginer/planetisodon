use worker::{Request, Response, Result, RouteContext};

use crate::{utils, Ctx};

pub async fn route_setting_txt(_: Request, ctx: RouteContext<Ctx>) -> Result<Response> {
    let board_key = ctx.param("boardKey").unwrap();

    let board = ctx
        .data
        .bbs_repository
        .get_board(board_key)
        .await
        .unwrap()
        .unwrap();
    let (title, default_name) = (board.name, "スケスケの名無し");
    let setting_txt = format!(
        "BBS_TITLE={title}
BBS_TITLE_ORIG={title}
BBS_NONAME_NAME={default_name}"
    );

    utils::response_shift_jis_text_plain(&setting_txt)
}
