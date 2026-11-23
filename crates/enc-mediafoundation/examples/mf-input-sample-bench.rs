fn main() {
    #[cfg(windows)]
    {
        if let Err(error) = cap_enc_mediafoundation::video::run_mf_input_sample_bench() {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
    #[cfg(not(windows))]
    {
        eprintln!("mf-input-sample-bench runs on Windows");
        std::process::exit(1);
    }
}
