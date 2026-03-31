use super::*;
    use crate::compress;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[test]
    fn test_decompress_file_not_exist() {
        let flags = Flags {
            quiet: false,
            verbose: true,
            force: false,
            sequitur: false,
            bisection: false,
            no_huffman: false,
        };
        let path = Path::new("wrong_path.gps");
        assert_eq!(run(path, flags), 2);
    }

    #[test]
    fn test_decompress_detect_wrong_header() {
        let mut file = NamedTempFile::new().unwrap();
        let content = b"Testing".to_vec();
        file.write_all(&content).unwrap();
        let path = file.path();
        let flags = Flags {
            quiet: false,
            verbose: true,
            force: false,
            sequitur: false,
            bisection: false,
            no_huffman: false,
        };
        assert_eq!(run(&path, flags), 2);
    }

    #[test]
    fn test_decompress_gps_header_invalid_size() {
        let mut file = NamedTempFile::new().unwrap();
        let header = b"GPS".to_vec();
        file.write_all(&header).unwrap();
        let path = file.path();
        let flags = Flags {
            quiet: false,
            verbose: true,
            force: false,
            sequitur: false,
            bisection: false,
            no_huffman: false,

        };
        assert_eq!(run(path, flags), 2);
    }

    #[test]
    fn test_compress_gps_header_correct() {
        let mut input_file = NamedTempFile::new().unwrap();
        writeln!(input_file, "Hello World!").unwrap();
        let input_path = input_file.path().to_path_buf();

        let flags = Flags {
            quiet: false,
            verbose: true,
            force: false,
            sequitur: false,
            bisection: false,
            no_huffman: false,
        };
        compress::run(&input_path, flags);
        let mut gps_path = input_path.clone();
        gps_path.set_file_name(format!(
            "{}.gps",
            input_path.file_name().unwrap().to_string_lossy()
        ));
        assert!(gps_path.exists());

        let decomp_flags = Flags {
            quiet: false,
            verbose: true,
            force: true,
            sequitur: false,
            bisection: false,
            no_huffman: false,
        };

        let decomp_result = run(&gps_path, decomp_flags);
        assert_eq!(decomp_result, 0);
        if gps_path.exists() {
            fs::remove_file(&gps_path).unwrap();
        }
    }

    #[test]
    fn test_list_file_not_exist() {
        let path = Path::new("wrong_path.gps");
        assert_eq!(list(path), 2);
    }

    #[test]
    fn test_list_detect_wrong_header() {
        let mut file = NamedTempFile::new().unwrap();
        let content = b"Testing".to_vec();
        file.write_all(&content).unwrap();
        let path = file.path();
        assert_eq!(list(&path), 2);
    }

    #[test]
    fn test_list_gps_header_invalid_size() {
        let mut file = NamedTempFile::new().unwrap();
        let header = b"GPS".to_vec();
        file.write_all(&header).unwrap();
        let path = file.path();
        assert_eq!(list(path), 2);
    }

    #[test]
    fn test_list_gps_header_correct() {
        let mut file = NamedTempFile::new().unwrap();
        let mut header = b"GPS".to_vec();
        let fake_size: u64 = 42;
        header.extend_from_slice(&fake_size.to_le_bytes());
        file.write_all(&header).unwrap();
        let path = file.path();
        let result = list(path);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_file_is_decompressed() {
        let mut file = NamedTempFile::new().unwrap();
        let content = b"Testing".to_vec();
        file.write_all(&content).unwrap();
        let path = file.path();
        let flags = Flags {
            quiet: false,
            verbose: true,
            force: false,
            sequitur: false,
            bisection: false,
            no_huffman: false,
        };
        compress::run(&path.to_path_buf(), flags);
        let compressed_path = path.with_extension("gps");

        let flags = Flags {
            quiet: false,
            verbose: true,
            force: true,
            sequitur: false,
            bisection: false,

            no_huffman: false,
        };
        let exit_code = run(&compressed_path, flags);

        let content = fs::read(path).unwrap();

        assert_eq!(String::from_utf8_lossy(&content), "Testing");
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_failed_to_overwrite() {
        let mut file = NamedTempFile::new().unwrap();
        let content = b"Testing".to_vec();
        file.write_all(&content).unwrap();
        let path = file.path();
        let flags = Flags {
            quiet: false,
            verbose: true,
            force: false,
            sequitur: false,
            bisection: false,

            no_huffman: false,
        };
        compress::run(&path.to_path_buf(), flags);
        let compressed_path = path.with_extension("gps");

        let flags = Flags {
            quiet: false,
            verbose: true,
            force: false,
            sequitur: false,
            bisection: false,
            no_huffman: false,
        };
        let exit_code = run(&compressed_path, flags);

        assert_eq!(exit_code, 2);
    }

    #[test]
    fn test_list_nonexistent_file() {
        let temp_path = PathBuf::from("notAPath.gps");
        let result = list(&temp_path);
        assert_eq!(result, 2);
    }