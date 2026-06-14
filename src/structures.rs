use crate::{DEV, ORG, TAG};

#[derive(serde::Deserialize)]
pub struct Token {
    pub token: String,
}

#[derive(serde::Deserialize, Debug)]
struct PushedData {
    pusher: String,
    tag: String,
}

#[derive(serde::Deserialize, Debug)]
struct Repository {
    is_private: bool,
    name: String,
    namespace: String, //org
    repo_name: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct DockerPayload {
    push_data: PushedData,
    repository: Repository,
}
impl DockerPayload {
    pub fn is_deployble(&self) -> bool {
        self.push_data.tag == TAG
            && self.push_data.pusher == DEV
            && self.repository.is_private
            && self.repository.namespace == ORG
            && self.repository.repo_name.starts_with(ORG)
    }
    pub fn check_repo_name(&self, name: &str) -> bool {
        self.repository.name == name
    }
}

pub struct Bashinfo<'a> {
    pub project: &'a str,
    pub path: &'a str,
    pub shahash: &'a str,
}
