use std::fs;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};
use async_ftp::{FtpError, FtpStream};
use async_ftp::types::FileType;
use async_recursion::async_recursion;
use tokio::fs::File;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    host: String,
    port: u16,
    user: String,
    password: String,
}

pub struct Uploader {
    stream: FtpStream,
    local_dir_count: usize,
    remote_dir: PathBuf,
}

impl Uploader {
    fn covert_path(p: &str) -> String {
        p.replace(MAIN_SEPARATOR, "/")
    }

    pub async fn new(settings: &Settings) -> Result<Self, FtpError> {
        let mut stream = FtpStream::connect(
            format!("{}:{}", settings.host, settings.port)
        ).await?;

        let _ = stream.login(
            settings.user.as_str(), settings.password.as_str()
        ).await?;

        stream.transfer_type(FileType::Binary).await?;

        Ok(Self {
            stream: stream,
            local_dir_count: 0,
            remote_dir: PathBuf::new(),
        })
    }

    pub async fn quit(&mut self) {
        let _ = self.stream.quit().await;
    }

    pub async fn mkdir(&mut self, remote: &str) {
        let converted = Self::covert_path(remote);
        let _ = self.stream.mkdir(converted.as_str()).await;
    }

    pub async fn file(&mut self, local: &str, remote: &str) {
        let mut file = File::open(local).await.unwrap();
        let converted = Self::covert_path(remote);
        let r = self.stream.put(converted.as_ref(), &mut file).await;
        if r.is_ok() {
            println!("{:?} -> {:?}", local, converted);
        } else {
            let mut remote_dirs = Path::new(converted.as_str()).to_path_buf();
            remote_dirs.pop();
            let mut dirs = PathBuf::new();
            for dir in &remote_dirs {
                dirs.push(dir);
                self.mkdir(dirs.to_str().unwrap()).await;
            }
            let _ = self.stream.put(converted.as_ref(), &mut file).await;
            println!("{:?} -> {:?}", local, converted);
        }
    }

    pub async fn dir(&mut self, local: &str, remote: &str) {
        let path: &Path = local.as_ref();
        self.local_dir_count = path.to_path_buf().iter().count();
        self.remote_dir.clear();
        self.remote_dir.push(remote);
        self.visit(path).await;
    }

    pub async fn files(&mut self, files: &Vec<String>, remote_base_dir: &str) {
        for file in files {
            let mut remote_path = Path::new(remote_base_dir).to_path_buf();
            remote_path.push(file);
            let remote_str = remote_path.to_str().unwrap();

            let local_path: &Path = file.as_ref();
            let local_str = local_path.to_str().unwrap();
            if local_path.is_dir() {
                self.dir(local_str, remote_str).await;
            } else if local_path.is_file() {
                self.file(local_str, remote_str).await;
            } else {
                eprintln!("unknown path: {}", local_str);
            }
        }
    }

    #[async_recursion]
    async fn visit(&mut self, dir: &Path) {
        if !dir.is_dir() {
            return;
        }

        let mut remote_path = self.remote_dir.clone();
        let mut index = 0;
        for d in dir.to_path_buf().iter() {
            if index >= self.local_dir_count {
                remote_path.push(d);
            }
            index += 1;
        }

        if remote_path.into_iter().count() > 0 {
            self.mkdir(remote_path.to_str().unwrap()).await;
        }

        for entry in fs::read_dir(dir).unwrap() {
            let dir = entry.unwrap();
            let path = dir.path();
            if path.is_dir() {
                self.visit(&path).await;
            } else {
                let remote_file = remote_path.join(path.file_name().unwrap());
                self.file(path.to_str().unwrap(), remote_file.to_str().unwrap()).await;
            }
        }
    }
}
