use bdrive::fs::LocalFile;

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    use std::fs::{read_to_string};
    use bdrive::bdrive::UploadOptions;
    use bdrive::conf::Configs;
    use bdrive::bdrive::BDrive;

    println!("creating bdrive...");

    let configs: Configs = toml::from_str(&read_to_string("config.toml")?).expect("AAAAAAAAH");
    println!("{:?}", configs);

    let mut bd = BDrive::new(configs).await?;

    let options = Some(UploadOptions::builder().overwrite(true).build());

    // upload this code for testing out the scan dir
    for f in bd.scan_dir("src").unwrap() {
        println!("=> Found: {:?}", f);
        match f {
            Ok(f) => {
                println!("=> Is file, hashing it...");
                let f = f.hash().unwrap();
                println!("=> and then upload it!");
                println!("=> DONE! (result: {:?})", bd.upload(f, options.clone()).await);
            }
            Err(e) => {
                println!("=> Not a file: {:?}", e);
            }
        }
    }


    Ok(())
}