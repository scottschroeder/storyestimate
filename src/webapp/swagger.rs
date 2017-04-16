use errors::*;
use rocket;
use rocket::Route;
use rocket::config::ConfigError;
use rocket::response::{NamedFile, Redirect};
use rocket_contrib::Template;
use std::env;
use std::path::PathBuf;
use webapp::filelike::FileLike;
use webapp::proxydata::ProxyData;


const SWAGGER_DEFAULT_DIR: &'static str = "vendor/swagger-ui";

pub fn routes() -> Vec<Route> {
    return routes![
        swagger_ui_home,
        swagger_ui,
    ];
}

#[derive(Serialize)]
struct HostInfo {
    hostname_port: String,
    scheme: String,
}

#[get("/")]
fn swagger_ui_home() -> Redirect {
    Redirect::to("/docs/index.html")
}
#[get("/<file..>")]
//fn swagger_ui(file: PathBuf, http_data: ProxyData) -> Result<CORS<FileLike>> {
fn swagger_ui(file: PathBuf, http_data: ProxyData) -> Result<FileLike> {
    let templates = vec!["index.html", "swagger.yaml"];
    for t in templates {
        if PathBuf::from(t) == file {
            info!("Treating {:?} as a template", file.display());


            let context = HostInfo {
                hostname_port: http_data.http_host,
                scheme: http_data.scheme,
            };
            info!("Template {} for user host", context.hostname_port);
            return Ok(FileLike::Template(Some(Template::render(t, &context))).into());
        }
    }
    let config = rocket::config::active().ok_or(rocket::config::ConfigError::NotFound)?;
    let swagger_dir: PathBuf = match config.get_str("swagger_dir") {
        Ok(dir_str) => PathBuf::from(dir_str),
        Err(ConfigError::NotFound) => env::current_dir()?.join(SWAGGER_DEFAULT_DIR),
        Err(e) => {
            warn!("Bad config value for 'swagger_dir': {:?}", e);
            bail!(e);
        },
    };

    let fullpath = swagger_dir.join(file);
    info!("Full Path is: {:?}", fullpath.display());
    let flike = match NamedFile::open(fullpath).ok() {
        Some(file) => FileLike::NamedFile(Some(file)),
        None => FileLike::NamedFile(None),
    };
    Ok(flike.into())
}
