use planetscale_driver::{query, PSConnection};
use worker::Date;

use crate::dtos::{Board, Res, Thread, User};

#[derive(Debug, Clone)]
pub struct CreatingThread {
    pub board_id: i32,
    pub title: String,
    pub name: String,
    pub mail: String,
    pub body: String,
    pub date: String,
    pub author_id: String,
    pub ip_addr: String,
    pub user_hash: String,
}

#[derive(Debug, Clone)]
pub struct CreatingResponse {
    pub board_id: i32,
    pub thread_key: i64,
    pub name: String,
    pub mail: String,
    pub body: String,
    pub date: String,
    pub author_id: String,
    pub ip_addr: String,
    pub user_hash: String,
}

#[derive(Clone)]
pub struct BbsRepository {
    conn: PSConnection,
}

impl BbsRepository {
    pub fn new(conn: PSConnection) -> Self {
        Self { conn }
    }

    pub async fn get_boards(&self) -> anyhow::Result<Vec<Board>> {
        let results = query("SELECT * FROM boards;")
            .fetch_all::<Board>(&self.conn)
            .await;

        match results {
            Ok(boards) => Ok(boards),
            Err(e) => {
                if e.to_string().contains("No results found") {
                    Ok(Vec::new())
                } else {
                    Err(anyhow::anyhow!("Error: unknown DB error in get boards"))
                }
            }
        }
    }

    pub async fn get_board(&self, board_key: &str) -> anyhow::Result<Option<Board>> {
        let result = query("SELECT * FROM boards WHERE board_key = '$0' LIMIT 1;")
            .bind(board_key)
            .fetch_one::<Board>(&self.conn)
            .await;

        match result {
            Ok(board) => Ok(Some(board)),
            Err(e) => {
                if e.to_string().contains("No results found") {
                    Ok(None)
                } else {
                    Err(anyhow::anyhow!("Error: unknown DB error in get board"))
                }
            }
        }
    }

    pub async fn get_threads(&self, board_key: &str) -> anyhow::Result<Vec<Thread>> {
        let result = query(
            "SELECT * FROM threads WHERE board_id IN 
            (SELECT id FROM boards WHERE board_key = '$0') 
            ORDER BY update_unix_timestamp DESC;",
        )
        .bind(board_key)
        .fetch_all::<Thread>(&self.conn)
        .await;

        match result {
            Ok(threads) => Ok(threads),
            Err(e) => {
                if e.to_string().contains("No results found") {
                    Ok(Vec::new())
                } else {
                    Err(anyhow::anyhow!("Error: unknown DB error in get threads"))
                }
            }
        }
    }

    pub async fn get_thread(&self, board_id: i32, thread_key: i64) -> anyhow::Result<Thread> {
        query("SELECT * FROM threads WHERE thread_key = $0 AND board_id = $1")
            .bind(thread_key)
            .bind(board_id)
            .fetch_one::<Thread>(&self.conn)
            .await
            .map_err(|e| {
                if e.to_string().contains("No results found") {
                    anyhow::anyhow!("Error: No results found in get thread")
                } else {
                    anyhow::anyhow!("Error: unknown DB error in get thread")
                }
            })
    }

    pub async fn get_thread_with_responses(
        &self,
        board_key: &str,
        thread_key: i64,
    ) -> anyhow::Result<(Thread, Vec<Res>)> {
        let thread = query(
            "SELECT * FROM threads WHERE thread_key = $0 AND board_id IN
        (SELECT id FROM boards WHERE board_key = '$1');",
        )
        .bind(thread_key)
        .bind(board_key)
        .fetch_one::<Thread>(&self.conn)
        .await
        .map_err(|e| {
            if e.to_string().contains("No results found") {
                anyhow::anyhow!("No results found")
            } else {
                anyhow::anyhow!("Error: unknown DB error in get thread with responses")
            }
        })?;

        let responses = query("SELECT * FROM responses WHERE thread_id = '$0';")
            .bind(&thread.id)
            .fetch_all::<Res>(&self.conn)
            .await;

        let responses = match responses {
            Ok(responses) => responses,
            Err(e) => {
                if e.to_string().contains("No results found") {
                    Vec::new()
                } else {
                    return Err(anyhow::anyhow!(
                        "Error: unknown DB error in get thread with responses"
                    ));
                }
            }
        };

        Ok((thread, responses))
    }

    pub async fn get_user(&self, user_hash: &str) -> anyhow::Result<Option<User>> {
        let result = query("SELECT * FROM users WHERE user_hash = '$0' LIMIT 1;")
            .bind(user_hash)
            .fetch_one::<User>(&self.conn)
            .await;

        match result {
            Ok(user) => Ok(Some(user)),
            Err(e) => {
                if e.to_string().contains("No results found") {
                    Ok(None)
                } else {
                    Err(anyhow::anyhow!("Error: unknown DB error in get user"))
                }
            }
        }
    }

    pub async fn create_thread(&self, thread: CreatingThread) -> anyhow::Result<()> {
        let thread_key = Date::now().as_millis() / 1000;
        let thread_id = uuid::Uuid::new_v7(uuid::Timestamp::from_unix(
            uuid::NoContext,
            Date::now().as_millis() / 1000,
            ((Date::now().as_millis() % 1000) * 1000) as u32,
        ));

        query(
            "INSERT INTO threads 
            (thread_key, board_id, title, ip_address, user_id, update_unix_timestamp, id, author_id)
        VALUES ($0, $1, '$2', '$3', '$4', $5, '$6', '$7')",
        )
        .bind(thread_key)
        .bind(thread.board_id)
        .bind(thread.title)
        .bind(&thread.ip_addr)
        .bind(&thread.user_hash)
        .bind(thread_key)
        .bind(thread_id)
        .bind(&thread.author_id)
        .execute(&self.conn)
        .await
        .map_err(|_| anyhow::anyhow!("Error: failed to insert thread"))?;

        let response_id = uuid::Uuid::new_v7(uuid::Timestamp::from_unix(
            uuid::NoContext,
            Date::now().as_millis() / 1000,
            (((Date::now().as_millis() + 1) % 1000) * 1000) as u32,
        ));
        query(
            "INSERT INTO responses 
            (thread_id, name, mail, body, author_id, date_text, ip_address, user_id, id)
        VALUES ('$0', '$1', '$2', '$3', '$4', '$5', '$6', '$7', '$8');",
        )
        .bind(thread_id)
        .bind(thread.name)
        .bind(thread.mail)
        .bind(thread.body)
        .bind(thread.author_id)
        .bind(thread.date)
        .bind(thread.ip_addr)
        .bind(thread.user_hash)
        .bind(response_id)
        .execute(&self.conn)
        .await
        .map_err(|_| anyhow::anyhow!("Error: failed to insert response"))?;

        Ok(())
    }

    pub async fn create_response(&self, response: CreatingResponse) -> anyhow::Result<()> {
        let thread = self
            .get_thread(response.board_id, response.thread_key)
            .await?;

        query("UPDATE threads SET response_count = response_count + 1 WHERE id = '$0';")
            .bind(&thread.id)
            .execute(&self.conn)
            .await
            .map_err(|_| anyhow::anyhow!("Error: failed to update response count of thread"))?;

        let response_id = uuid::Uuid::new_v7(uuid::Timestamp::from_unix(
            uuid::NoContext,
            Date::now().as_millis() / 1000,
            ((Date::now().as_millis() % 1000) * 1000) as u32,
        ));
        query(
            "INSERT INTO responses 
            (thread_id, name, mail, body, author_id, date_text, ip_address, user_id, id)
        VALUES ('$0', '$1', '$2', '$3', '$4', '$5', '$6', '$7', '$8');",
        )
        .bind(&thread.id)
        .bind(response.name)
        .bind(response.mail)
        .bind(response.body)
        .bind(response.author_id)
        .bind(response.date)
        .bind(response.ip_addr)
        .bind(response.user_hash)
        .bind(response_id)
        .execute(&self.conn)
        .await
        .map_err(|_| anyhow::anyhow!("Error: failed to insert response"))?;

        Ok(())
    }
}
