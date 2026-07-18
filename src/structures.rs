use crate::{DEV, ORG, TAG};
use serde::Deserialize;
use std::collections::HashMap;

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
        let org = ORG.get().unwrap();
        &self.push_data.tag == TAG.get().unwrap()
            && &self.push_data.pusher == DEV.get().unwrap()
            && self.repository.is_private
            && &self.repository.namespace == org
            && self.repository.repo_name.starts_with(org)
    }
    pub fn check_repo_name(&self, name: &str) -> bool {
        self.repository.name == name
    }
}

#[derive(Debug, Deserialize)]
pub struct Main {
    #[serde(rename = "DEV")]
    pub dev: String,
    #[serde(rename = "ORG")]
    pub org: String,
    #[serde(rename = "TAG")]
    pub tag: String,
}
#[derive(Debug, Deserialize)]
pub struct Config {
    pub main: Main,
    #[serde(rename = "R")]
    pub projects: Option<HashMap<String, Bashinfo>>,
}
#[derive(Debug, Deserialize)]
pub struct Bashinfo {
    pub repo: String,
    pub path: String,
    pub shahash: String,
}
impl Bashinfo {
    pub fn get_repo(&self) -> &str {
        &self.repo
    }
    pub fn get_path(&self) -> &str {
        &self.path
    }
}
