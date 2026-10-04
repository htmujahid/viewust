use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct UserRow {
    pub(crate) name: String,
    pub(crate) uid: String,
    pub(crate) primary_group: String,
    pub(crate) groups: Vec<String>,
    pub(crate) full_name: Option<String>,
    pub(crate) home: Option<String>,
    pub(crate) shell: Option<String>,
    pub(crate) kind: &'static str,
    pub(crate) admin: bool,
    pub(crate) can_login: Option<bool>,
    pub(crate) current: bool,
    pub(crate) processes: u32,
    pub(crate) memory: u64,
}

#[derive(Serialize, Clone)]
pub struct GroupRow {
    pub(crate) name: String,
    pub(crate) gid: String,
    pub(crate) members: Vec<String>,
    pub(crate) kind: &'static str,
    pub(crate) admin: bool,
}

#[derive(Serialize)]
pub struct Accounts {
    pub(crate) users: Vec<UserRow>,
    pub(crate) groups: Vec<GroupRow>,
}
