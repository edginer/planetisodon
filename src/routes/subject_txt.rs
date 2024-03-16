use worker::{Cache, Request, Response, Result, RouteContext};

use crate::{dtos::Thread, utils, Ctx};

pub async fn route_subject_txt(req: Request, ctx: RouteContext<Ctx>) -> Result<Response> {
    let cache = Cache::default();

    let is_mate = req
        .headers()
        .get("User-Agent")
        .ok()
        .flatten()
        .unwrap_or_default()
        .contains("chMate")
        || req
            .headers()
            .get("X-ThreadList-AuthorId-Supported")
            .ok()
            .flatten()
            .unwrap_or_default()
            .trim()
            == "true";

    if is_mate {
        let req_mate = req.clone();
        if let Ok(mut req_mate) = req_mate {
            let t = req_mate.path_mut();
            if let Ok(t) = t {
                *t = t.replace("subject.txt", "subject_mate.txt");
                if let Ok(Some(s)) = cache.get(&req_mate, false).await {
                    return Ok(s);
                }
            }
        }
    } else if let Ok(Some(s)) = cache.get(&req, false).await {
        return Ok(s);
    }

    let board_key = ctx.param("boardKey").unwrap();

    let threads = ctx
        .data
        .bbs_repository
        .get_threads(board_key)
        .await
        .unwrap();

    let subject_txt = gen_subject_txt(&threads);
    let mate_subject_txt = gen_mate_subject_txt(&threads);

    let mut data = utils::response_shift_jis_text_plain_with_cache(&subject_txt, 1)?;
    let mut data_mate = utils::response_shift_jis_text_plain_with_cache(&mate_subject_txt, 1)?;
    let ret_data = if is_mate {
        data_mate.cloned()
    } else {
        data.cloned()
    };
    if let Ok(ret_data) = ret_data {
        if ret_data.status_code() == 200 {
            let req_mate = req.clone();
            if let Ok(mut req_mate) = req_mate {
                let t = req_mate.path_mut();
                if let Ok(t) = t {
                    *t = t.replace("subject.txt", "subject_mate.txt");
                    let _ = cache.put(&req_mate, data_mate).await;
                }
            }
            let _ = cache.put(&req, data).await;
        }
        Ok(ret_data)
    } else {
        Ok(if is_mate { data_mate } else { data })
    }
}

fn gen_subject_txt(threads: &[Thread]) -> String {
    let mut subject_txt = String::new();
    for thread in threads {
        subject_txt.push_str(&format!(
            "{}.dat<>{} ({})\n",
            thread.thread_key, thread.title, thread.response_count
        ));
    }
    subject_txt
}

fn gen_mate_subject_txt(threads: &[Thread]) -> String {
    let mut subject_txt = String::new();
    for thread in threads {
        subject_txt.push_str(&format!(
            "{}.dat<>{} [{}★] ({})\n",
            thread.thread_key, thread.title, thread.author_id, thread.response_count
        ));
    }
    subject_txt
}
