use planetscale_driver::Database;

#[derive(Debug, Database)]
pub struct Board {
    pub id: i32,
    pub name: String,
    pub board_key: String,
    pub default_name: String,
}

#[derive(Debug, Database)]
pub struct Thread {
    pub id: String,
    pub thread_key: i64,
    pub board_id: i32,
    pub title: String,
    pub response_count: i32,
    pub ip_address: String,
    pub user_id: String,
    pub created_at: String,
    pub update_unix_timestamp: i64,
    pub author_id: String,
}

#[derive(Debug, Database)]
pub struct Res {
    pub id: String,
    pub thread_id: String,
    pub name: String,
    pub mail: String,
    pub body: String,
    pub author_id: String,
    pub date_text: String,
    pub ip_address: String,
    pub user_id: String,
    pub created_at: String,
}

#[derive(Debug, Database)]
pub struct User {
    pub id: String,
    pub ip_address: String,
    pub created_at: String,
    pub disabled: i32,
    pub user_hash: String,
}
