use std::fs::{read_to_string};
use bdrive::bdrive::UploadOptions;

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    // let f1 = File::try_from("test.iso".to_string())?;
    // let f2 = File::try_from("test.iso".to_string())?;
    // println!("hashing f1");
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

    // for i in ["test.iso", "/documents/rust/bdrive/test2.iso"] {
    //     let f = bd.load_file(i).unwrap();
    //     let f = f.hash().unwrap();
    //     println!("{:?}", bd.upload(f, )).await);
    // }

    let overwrite = Some(UploadOptions::builder().overwrite(true).build());

    for f in bd.scan_dir("src").unwrap() {
        println!("=> Found: {:?}", f);
        match f {
            Ok(f) => {
                println!("=> Is file, hashing it...");
                let f = f.hash().unwrap();
                println!("=> and then upload it!");
                println!("=> DONE! (result: {:?})", bd.upload(f, overwrite.clone()).await);
            }
            Err(e) => {
                println!("=> Not a file: {:?}", e);
            }
        }
    }


    return Ok(());
}