mod api;
mod cli;
mod log;

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let parsed = match cli::parse_args(&args) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("error: {e}");
            log::write_log(&format!("{} [ERR] {e}", log::now_str()));
            process::exit(1);
        }
    };

    let cli::ParseResult::Args(args) = parsed else {
        cli::print_help();
        process::exit(0);
    };

    let app_id = match env::var("MPUSH_APP_ID") {
        Ok(v) if !v.is_empty() => v,
        _ => {
            eprintln!("error: missing env MPUSH_APP_ID");
            log::write_log(&format!("{} [ERR] missing_env var=MPUSH_APP_ID", log::now_str()));
            process::exit(1);
        }
    };

    let app_secret = match env::var("MPUSH_APP_SECRET") {
        Ok(v) if !v.is_empty() => v,
        _ => {
            eprintln!("error: missing env MPUSH_APP_SECRET");
            log::write_log(&format!("{} [ERR] missing_env var=MPUSH_APP_SECRET", log::now_str()));
            process::exit(1);
        }
    };

    let access_token = match api::get_access_token(&app_id, &app_secret) {
        Ok(token) => token,
        Err(e) => {
            eprintln!("error: {e}");
            log::write_log(&format!("{} [ERR] {e}", log::now_str()));
            process::exit(1);
        }
    };

    let masked_user = log::mask(&args.user);
    let masked_tmpl = log::mask(&args.template);

    match api::send_template_message(
        &access_token,
        &args.user,
        &args.template,
        args.link.as_deref(),
        &args.data,
    ) {
        Ok(msgid) => {
            log::write_log(&format!(
                "{} [OK] user={masked_user} template={masked_tmpl} msgid={msgid}",
                log::now_str()
            ));
        }
        Err(e) => {
            eprintln!("error: {e}");
            log::write_log(&format!(
                "{} [ERR] user={masked_user} template={masked_tmpl} {e}",
                log::now_str()
            ));
            log::cleanup_logs();
            process::exit(1);
        }
    }

    log::cleanup_logs();
}
