# FTP upload cmd

## Settings

* create configuration file(YAML)
    ```yaml
    host: {target host ip}
    port: {ftp port}
    user: {user name}
    password: {user password}
    ```

## Usage

```
up-ftp -f {configuration_file} -d {remote_base_dir} files..
```

