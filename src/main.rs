use std::fs::File;
use std::path::Path;
use std::rc::Rc;

use clipboard::{ClipboardContext, ClipboardProvider};
use reqwest::{Error, header::{DATE, HeaderMap}};
use slint::{Model, ModelRc, SharedString, VecModel};

slint::include_modules!();

const CREATED: u16 = 201;
fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    // Done
    ui.on_start_upload({
            let ui_handle = ui.as_weak();
            move |files: ModelRc<SharedString>| {
                if let Some(ui) = ui_handle.upgrade() {

                    let subdomain: SharedString = ui.get_subdomain();
                    let container: SharedString = ui.get_container();
                    let token: SharedString = ui.get_token();

                    for file in files.iter() {

                        let file_name = Path::new(file.as_str())
                            .file_name()
                            .unwrap()
                            .to_str()
                            .clone()
                            .unwrap();

                        let full_url = format!("https://{}.blob.core.windows.net/{}/{}", subdomain, container, &file_name);

                        let as_file_buf = File::open(format!("{}", file.as_str()));

                        if let Ok(_) = as_file_buf {
                            let response = upload_file(&full_url, token.as_str().into(), file.as_str());
                            if let Ok(status_code) = response
                            {
                                match status_code
                                {
                                    CREATED  => {
                                        let prev_urls: ModelRc<SharedString> = ui.get_generated_urls();
                                        let mut items: Vec<SharedString> = prev_urls.iter().collect();
                                        items.push(full_url.clone().into());

                                        let rc_vec_model: Rc<VecModel<SharedString>> = Rc::new(VecModel::from(items.clone()));
                                        let generated_urls: ModelRc<SharedString> = ModelRc::from(rc_vec_model.clone());

                                        ui.set_generated_urls(generated_urls);
                                    }

                                    other => {
                                        let prev_urls: ModelRc<SharedString> = ui.get_generated_urls();
                                        let mut items: Vec<SharedString> = prev_urls.iter().collect();
                                        items.push(format!("{} -> {:?}", <String as Into<SharedString>>::into(full_url.clone()), other).into());

                                        let rc_vec_model: Rc<VecModel<SharedString>> = Rc::new(VecModel::from(items.clone()));
                                        let generated_urls: ModelRc<SharedString> = ModelRc::from(rc_vec_model.clone());

                                        ui.set_generated_urls(generated_urls);
                                    }
                                }
                            }
                            else if let Err(err_code) = response
                            {
                                let prev_urls: ModelRc<SharedString> = ui.get_generated_urls();
                                let mut items: Vec<SharedString> = prev_urls.iter().collect();
                                items.push(format!("{} -> {:?}", <String as Into<SharedString>>::into(full_url.clone()), <String as Into<SharedString>>::into(err_code.to_string())).into());

                                let rc_vec_model: Rc<VecModel<SharedString>> = Rc::new(VecModel::from(items.clone()));
                                let generated_urls: ModelRc<SharedString> = ModelRc::from(rc_vec_model.clone());

                                ui.set_generated_urls(generated_urls);
                            }
                        }
                        else if let Err(err_response) = as_file_buf {
                            let prev_urls: ModelRc<SharedString> = ui.get_generated_urls();
                            let mut items: Vec<SharedString> = prev_urls.iter().collect();
                            items.push(format!("{} -> {:?}", <String as Into<SharedString>>::into(full_url.clone()), <String as Into<SharedString>>::into(err_response.to_string())).into());

                            let rc_vec_model: Rc<VecModel<SharedString>> = Rc::new(VecModel::from(items.clone()));
                            let generated_urls: ModelRc<SharedString> = ModelRc::from(rc_vec_model.clone());

                            ui.set_generated_urls(generated_urls);
                        }
                    }
                }
            }
        });
    // Done
    ui.on_load_files({
        let ui_handle = ui.as_weak();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                let files = rfd::FileDialog::new()
                    .set_title("Pilih file html nya")
                    .pick_files();
                if let Some(files) = files {
                    // Convert file paths to SharedString and update the property
                    let selected_files: Vec<SharedString> = files
                        .into_iter()
                        .map(|path| SharedString::from(path.to_string_lossy().into_owned()))
                        .collect();

                    let model: Rc<VecModel<SharedString>> = Rc::new(VecModel::from(selected_files));
                    ui.set_file_lists(ModelRc::from(model));
                }
            }
        }
    });
    // DONE
    ui.on_send_to_clipboard({
        let ui_handle = ui.as_weak();
        move || {
            if let Some(ui) = ui_handle.upgrade() {
                let mut raw_urls: Vec<String> = Vec::new();
                ui.get_generated_urls()
                    .iter()
                    .for_each(|val| raw_urls.push(val.to_string()));

                let urls_string = raw_urls.join("\n");

                let mut ctx: ClipboardContext = clipboard::ClipboardProvider::new().unwrap();
                ctx.set_contents(urls_string.clone()).unwrap();
            }
        }
    });
    ui.run()
}

fn upload_file(url: &String, token: &str, path_to_file: &str) -> Result<u16, Error> {
    let body: File = File::open(path_to_file).unwrap();

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", token.parse().unwrap() );
    headers.insert("x-ms-date", DATE.into());
    headers.insert("x-ms-version", "2024-05-25".parse().unwrap());

    let client = reqwest::blocking::Client::new();
    let res = client.put(url)
        .headers(headers)
        .body(body)
        .send()?;

    Ok(res.status().as_u16())
}
