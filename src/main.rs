use std::rc::Rc;

use reqwest::{Error, header::{DATE, HeaderMap}};
use slint::{Model, ModelRc, SharedString, VecModel};

slint::include_modules!();

const RES_TYPE: &str = "restype=container";
const CREATED: u16 = 201; // Use a constant for the created status code

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    ui.on_start_upload({
            let ui_handle = ui.as_weak();
            move |files: ModelRc<SharedString>| {
                if let Some(ui) = ui_handle.upgrade() {

                    let subdomain: SharedString = ui.get_subdomain();
                    let container: SharedString = ui.get_container();
                    let token: SharedString = ui.get_token();

                    for file in files.iter() {
                        let full_url: String = format!("https://{subdomain}.blob.core.windows.net/{container}/{file}?restype={RES_TYPE}");
                        let file_buf = std::fs::File::open(format!("{}", file.as_str()));

                        if let Ok(file_res) = file_buf {
                            let response = upload_file(&full_url, &format!("{token}"), file_res);
                            if let Ok(status_code) = response
                            {
                                match status_code
                                {
                                    CREATED =>
                                        {
                                            let prev_urls: ModelRc<SharedString> = ui.get_generated_urls();
                                            let mut items: Vec<SharedString> = prev_urls.iter().collect();
                                            items.push(full_url.into());

                                            let rc_vec_model: Rc<VecModel<SharedString>> = Rc::new(VecModel::from(items.clone()));
                                            let generated_urls: ModelRc<SharedString> = ModelRc::from(rc_vec_model.clone());

                                            ui.set_generated_urls(generated_urls);
                                        }

                                    other =>
                                        {
                                            let prev_urls: ModelRc<SharedString> = ui.get_generated_urls();
                                            let mut items: Vec<SharedString> = prev_urls.iter().collect();
                                            items.push(format!("{} -> {:?}", <String as Into<SharedString>>::into(full_url), other).into());

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
                        else if let Err(err_response)  = file_buf {
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

    ui.run()
}

async fn upload_file(url: &String, token: &String, body: std::fs::File) -> Result<u16, Error> {
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", format!("SharedKey myaccount:{token:?}").parse().unwrap());
    headers.insert("x-ms-date", DATE.into());
    headers.insert("x-ms-version", "2024-05-25".parse().unwrap());

    let client = reqwest::blocking::Client::new();
    let res = client.put(url)
        .headers(headers)
        .body(body)
        .send()?;

    println!("Response: {:?}", res);
    Ok(res.status().as_u16())
}
