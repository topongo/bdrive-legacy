use std::fs::{read_to_string, File as StdFile};
use std::io::Read;
use futures::TryFutureExt;
use mongodb::{options::{ClientOptions, ServerApi, ServerApiVersion}, Client};
use bdrive::bdrive::BDrive;
use bdrive::fs::{File, LocalFile};
use bdrive::db::Database;
use bdrive::fs::ToRemoteFile;

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    // let f1 = File::try_from("test.iso".to_string())?;
    let f2 = File::try_from("test.iso".to_string())?;
    println!("hashing f1");
    // let f1 = f1.hash().map_err(|(e, f)| {
    //     eprintln!("Error while hashing f ({:?}): ", f);
    //     e
    // })?;
    // println!("hashing f2");
    let f2 = f2.hash().map_err(|(e, f)| {
        eprintln!("Error while hashing f ({:?}): ", f);
        e
    })?;
    // println!("f1 == f2 : {}", f1 == f2);
    // println!("serialized:\n{}\n", serde_json::to_string(&f1.to_remote_file()).unwrap());

    use bdrive::conf::Configs;
    use bdrive::bdrive::BDrive;

    let configs: Configs = toml::from_str(&*read_to_string("config.toml")?).expect("AAAAAAAAH");
    println!("{:?}", configs);

    println!("creating bdrive...");

    let mut bd = BDrive::new(configs).await?;

    println!("{:?}", bd.upload(f2, None).await);

    return Ok(());

    // let mut bd = BDrive::connect(client.database("test")).await?;
    //
    // if bd.exists("test.iso").await? {
    //     println!("{:?}", bd.upload(
    //         f1,
    //         Some(
    //             UploadOptions::builder()
    //                 .overwrite(true)
    //                 .build()
    //         )).await);
    // } else {
    //     println!("file doesn't exists remotely, creating it.");
    //     println!("{:?}", bd.upload(f1, None).await);
    // }
    //
    // Ok(())
}