use mongodb::options::{ClientOptions, ServerApiVersion, ServerApi};
use mongodb::Client;
use crate::conf::MongoDBConfig;
use crate::db;

impl MongoDBConfig {
    pub async fn to_db(self) -> mongodb::error::Result<db::Database> {
        let uri = format!(
            "mongodb+srv://{}:{}@{}/?retryWrites=true&w=majority",
            self.username,
            self.password,
            self.host,
        );
        let mut client_options =
            ClientOptions::parse(uri)
                .await?;
        let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
        client_options.server_api = Some(server_api);
        let client = Client::with_options(client_options)?;
        Ok(db::Database::connect(client.database("bdrive")).await?)
    }
}
