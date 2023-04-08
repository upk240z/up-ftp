use std::{fs, process};
use ftp::{Uploader};
use getopt::Opt;

mod ftp;

#[tokio::main]
async fn main() {
    let mut args: Vec<String> = std::env::args().collect();
    let mut opts = getopt::Parser::new(&args, "f:d:");
    let mut yaml_file: Option<String> = None;
    let mut remote_base: Option<String> = None;

    loop {
        match opts.next().transpose().unwrap() {
            None => break,
            Some(opt) => match opt {
                Opt('f', Some(s)) => {
                    yaml_file = Some(s);
                },
                Opt('d', Some(s)) => {
                    remote_base = Some(s);
                },
                _ => unreachable!(),
            }
        }
    }

    let Some(yaml_file_path) = yaml_file else {
        eprintln!("Usage {} -f yaml_file -d remote_base_dir", args[0]);
        process::exit(1);
    };

    let Some(remote_base_dir) = remote_base else {
        eprintln!("Usage {} -f yaml_file -d remote_base_dir", args[0]);
        process::exit(1);
    };

    let Ok(yaml_string) = fs::read_to_string(yaml_file_path.as_str()) else {
        eprintln!("failed to read file: {}", yaml_file_path);
        process::exit(1);
    };

    let Ok(settings) = serde_yaml::from_str(yaml_string.as_str()) else {
        eprintln!("failed to parse yaml file");
        process::exit(1);
    };

    let files = args.split_off(opts.index());
    let mut uploader = Uploader::new(&settings).await;
    uploader.files(&files, remote_base_dir.as_str()).await;
    uploader.quit().await;
}
