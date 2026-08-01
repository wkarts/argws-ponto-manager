#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args = argws_ponto_manager_lib::cli::args::CliArgs::parse();
    if argws_ponto_manager_lib::cli::runner::should_run_without_tauri(&args) {
        if let Err(err) = argws_ponto_manager_lib::cli::runner::run(args) {
            eprintln!("{err}");
            std::process::exit(1);
        }
        return;
    }
    #[cfg(feature = "desktop")]
    argws_ponto_manager_lib::run();

    #[cfg(not(feature = "desktop"))]
    {
        eprintln!(
            "O modo desktop exige a feature Cargo 'desktop'. Use --headless, --cli ou --worker."
        );
        std::process::exit(2);
    }
}
