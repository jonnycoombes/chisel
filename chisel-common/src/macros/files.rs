/// Macro for some relative file shennigans
#[macro_export]
macro_rules! relative_file {
    ($f : expr) => {{
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        base.join($f)
    }};
}

/// Instantiate a file relative to the cargo manifest directory
#[macro_export]
macro_rules! file_from_relative_path {
    ($f : expr) => {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = base.join($f);
        let f = File::open(path).unwrap();
    };
}

/// Take some bytes and create a [BufRead]
#[macro_export]
macro_rules! reader_from_bytes {
    ($b : expr) => {{
        let buffer: &[u8] = $b.as_bytes();
        BufReader::new(buffer)
    }};
}

/// Grab all the lines from a file relative to the current directory
#[macro_export]
macro_rules! lines_from_relative_file {
    ($f : expr) => {{
        let path = env::current_dir().unwrap().join($f);
        let f = File::open(path).unwrap();
        BufReader::new(f).lines()
    }};
}
