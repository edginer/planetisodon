use worker::{Cache, Request, Response, Result, RouteContext};

use crate::{utils, Ctx};

pub async fn route_dat(req: Request, ctx: RouteContext<Ctx>) -> Result<Response> {
    let cache = Cache::default();
    if let Ok(Some(s)) = cache.get(&req, false).await {
        return Ok(s);
    }

    let board_key = ctx.param("boardKey").unwrap();
    let thread_key = ctx.param("threadKey").unwrap().replace(".dat", "");

    let (thread, responses) = ctx
        .data
        .bbs_repository
        .get_thread_with_responses(board_key, thread_key.parse().unwrap())
        .await
        .unwrap();

    let mut dat = String::new();
    for (idx, response) in responses.iter().enumerate() {
        dat.push_str(&format!(
            "{}<><>{} ID:{}<> {}<>{}\n",
            if response.name.is_empty() {
                "スケスケの名無し"
            } else {
                &response.name
            },
            response.date_text,
            response.author_id,
            response.body,
            if idx == 0 { &thread.title } else { "" }
        ));
    }

    let mut data = utils::response_shift_jis_text_plain_with_cache(&dat, 1)?;
    if let Ok(result) = data.cloned() {
        if result.status_code() == 200 {
            let _ = cache.put(&req, result).await;
        }
    }

    Ok(data)
}
