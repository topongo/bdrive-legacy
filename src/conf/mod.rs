mod mongodb;
mod ssh;
mod paths;

pub use paths::PathError;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Configs {
    pub ssh: SSHConfig,
    pub mongodb: MongoDBConfig,
    pub paths: PathsConf
}

#[derive(Deserialize, Debug)]
pub struct SSHConfig {
    pub host: String,
    pub port: u16,
    pub username: String
}

#[derive(Deserialize, Debug)]
pub struct MongoDBConfig {
    pub username: String,
    pub host: String,
    pub port: Option<u16>,
    pub password: String
}

#[derive(Deserialize, Debug)]
pub struct PathsConf {
    pub local: String,
    pub remote: String
}
