use std::fs::{read_to_string};

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    // let f1 = File::try_from("test.iso".to_string())?;
    // let f2 = File::try_from("test.iso".to_string())?;
    println!("hashing f1");
    // let f1 = f1.hash().map_err(|(e, f)| {
    //     eprintln!("Error while hashing f ({:?}): ", f);
    //     e
    // })?;
    // println!("hashing f2");
    // let f2 = f2.hash().map_err(|(e, f)| {
    //     eprintln!("Error while hashing f ({:?}): ", f);
    //     e
    // })?;
    // println!("f1 == f2 : {}", f1 == f2);
    // println!("serialized:\n{}\n", serde_json::to_string(&f1.to_remote_file()).unwrap());

    use bdrive::conf::Configs;
    use bdrive::bdrive::BDrive;
    use bodo_connect::net::NetworkMap;
    use bodo_connect::net::Subnet;

    let nmap = NetworkMap::try_from(serde_json::from_str::<Vec<Subnet>>(&*read_to_string("/home/topongo/.config/bodoConnect/networkmap.json")?).unwrap()).unwrap();
    nmap.wake(nmap.get_host("dell").unwrap()).await.unwrap();

    let configs: Configs = toml::from_str(&*read_to_string("config.toml")?).expect("AAAAAAAAH");
    println!("{:?}", configs);

    println!("creating bdrive...");

    let mut bd = BDrive::new(configs).await?;

    let f = bd.load_local_file("test.iso").unwrap();
    println!("CWD: {:?}", std::env::current_dir());
    let mut cwd = std::env::current_dir().unwrap();
    for i in ["documents", "rust", "bdrive"] {
        println!("{:?} exists? {}", cwd, cwd.exists());
        cwd.push(i);
    }
    let f = f.hash().unwrap();

    println!("{:?}", bd.upload(f, None).await);

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