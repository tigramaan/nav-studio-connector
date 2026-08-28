fn main() {
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    if args.len() > 1 {
        std::process::exit(nav_studio_connector_lib::cli::run_from(args));
    }
    #[cfg(feature = "desktop")]
    nav_studio_connector_lib::run();
    #[cfg(not(feature = "desktop"))]
    {
        eprintln!("GUI support is disabled in this build; use a CLI command such as 'agent describe --json'");
        std::process::exit(2);
    }
}
