#[cfg(target_os = "android")]
use jni::{
    objects::{JClass, JObject},
    JNIEnv,
};

#[cfg(not(feature = "test_utils"))]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use identity_wallet::{
        persistence::{clear_assets_tmp_folder, initialize_storage},
        state::AppStateContainer,
    };
    use log::{info, warn, LevelFilter};
    use tauri::Emitter;
    use tauri_plugin_log::{fern::colors::Color, fern::colors::ColoredLevelConfig, Target, TargetKind};

    #[cfg_attr(not(desktop), allow(unused_mut))]
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|_app, argv, _cwd| {
            info!("New app instance opened via deep link: {argv:?}");
        }));
    }

    builder
        .invoke_handler(tauri::generate_handler![
            tauri_command::handle_action,
            tauri_command::fetch_objectid_terms
        ])
        .setup(move |app| {
            info!("setting up tauri app");
            initialize_storage(app.handle()).ok();
            clear_assets_tmp_folder().ok();
            dotenvy::dotenv().ok();
            #[cfg(mobile)]
            {
                app.handle().plugin(tauri_plugin_barcode_scanner::init())?;
                app.handle().plugin(tauri_plugin_biometric::init())?;
                app.handle().plugin(tauri_plugin_keystore::init())?;
            }
            Ok(())
        })
        .manage(AppStateContainer(Default::default()))
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([Target::new(TargetKind::Stdout), Target::new(TargetKind::Webview)])
                .level(LevelFilter::Info)
                .level_for("objectid", LevelFilter::Debug)
                .level_for("identity_wallet", LevelFilter::Debug)
                .with_colors(
                    ColoredLevelConfig::new()
                        .trace(Color::White)
                        .debug(Color::Cyan)
                        .info(Color::Green)
                        .warn(Color::Yellow)
                        .error(Color::Red),
                )
                .build(),
        )
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if matches!(event, tauri::RunEvent::Resumed) {
                info!("application resumed");
                if let Err(error) = app_handle.emit("objectid-resumed", ()) {
                    warn!("failed to emit resume event: {error}");
                }
            }
        });
}

pub mod tauri_command {
    use identity_wallet::{
        command::Runtime,
        state::{actions::Action, AppStateContainer},
    };
    use std::time::Duration;

    #[tauri::command]
    pub async fn handle_action(
        action: Action,
        app_handle: tauri::AppHandle<Runtime>,
        container: tauri::State<'_, AppStateContainer>,
        window: tauri::Window<Runtime>,
    ) -> Result<(), String> {
        identity_wallet::command::handle_action(action, app_handle, container, window).await
    }

    #[tauri::command]
    pub async fn fetch_objectid_terms() -> Result<String, String> {
        const TERMS_URL: &str = "https://objectid.io/general-terms/";
        const TERMS_API_URL: &str =
            "https://objectid.io/wp-json/wp/v2/pages?slug=general-terms&_fields=title,content,modified,date";

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .user_agent("ObjectID Wallet")
            .build()
            .map_err(|error| error.to_string())?;

        if let Ok(response) = client.get(TERMS_API_URL).send().await {
            if response.status().is_success() {
                let body = response.text().await.map_err(|error| error.to_string())?;
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                    if let Some(page) = json.as_array().and_then(|pages| pages.first()) {
                        let mut rendered = String::new();
                        if let Some(title) = page.pointer("/title/rendered").and_then(|value| value.as_str()) {
                            rendered.push_str(title);
                            rendered.push_str("\n\n");
                        }
                        if let Some(date) = page
                            .get("modified")
                            .or_else(|| page.get("date"))
                            .and_then(|value| value.as_str())
                        {
                            rendered.push_str("Last updated: ");
                            rendered.push_str(date);
                            rendered.push_str("\n\n");
                        }
                        if let Some(content) = page.pointer("/content/rendered").and_then(|value| value.as_str()) {
                            rendered.push_str(content);
                        }

                        let terms = html_to_readable_text(&rendered);
                        if terms.len() > 100 {
                            return Ok(terms);
                        }
                    }
                }
            }
        }

        let body = client
            .get(TERMS_URL)
            .send()
            .await
            .map_err(|error| error.to_string())?
            .text()
            .await
            .map_err(|error| error.to_string())?;
        let terms = html_to_readable_text(&body);
        if terms.len() > 100 {
            Ok(terms)
        } else {
            Err("Unable to read ObjectID terms from the live page.".to_string())
        }
    }

    fn html_to_readable_text(html: &str) -> String {
        let mut html = strip_between_tags(html, "script");
        html = strip_between_tags(&html, "style");
        html = strip_between_tags(&html, "noscript");

        for tag in [
            "</h1>",
            "</h2>",
            "</h3>",
            "</h4>",
            "</p>",
            "</li>",
            "</div>",
            "</section>",
            "</article>",
            "<br>",
            "<br/>",
            "<br />",
        ] {
            html = html.replace(tag, "\n");
        }
        html = html.replace("<li", "\n- <li");

        let mut text = String::with_capacity(html.len());
        let mut inside_tag = false;
        for character in html.chars() {
            match character {
                '<' => inside_tag = true,
                '>' => inside_tag = false,
                _ if !inside_tag => text.push(character),
                _ => {}
            }
        }

        let decoded = decode_html_entities(&text);
        let mut lines = Vec::new();
        let mut previous_blank = false;
        for line in decoded.lines() {
            let collapsed = line.split_whitespace().collect::<Vec<_>>().join(" ");
            let trimmed = collapsed.trim();
            if trimmed.is_empty() {
                if !previous_blank {
                    lines.push(String::new());
                    previous_blank = true;
                }
                continue;
            }
            lines.push(trimmed.to_string());
            previous_blank = false;
        }

        lines.join("\n").trim().to_string()
    }

    fn strip_between_tags(input: &str, tag: &str) -> String {
        let mut output = input.to_string();
        let open_tag = format!("<{tag}");
        let close_tag = format!("</{tag}>");

        loop {
            let lower = output.to_lowercase();
            let Some(start) = lower.find(&open_tag) else {
                break;
            };
            let Some(relative_end) = lower[start..].find(&close_tag) else {
                break;
            };
            let end = start + relative_end + close_tag.len();
            output.replace_range(start..end, "");
        }

        output
    }

    fn decode_html_entities(input: &str) -> String {
        let input = input
            .replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&quot;", "\"")
            .replace("&#039;", "'")
            .replace("&apos;", "'")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&hellip;", "...")
            .replace("&ndash;", "-")
            .replace("&mdash;", "-");

        let mut output = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();
        while let Some(character) = chars.next() {
            if character != '&' || chars.peek() != Some(&'#') {
                output.push(character);
                continue;
            }

            chars.next();
            let is_hex = matches!(chars.peek(), Some('x' | 'X'));
            if is_hex {
                chars.next();
            }

            let mut entity = String::new();
            while let Some(next) = chars.peek() {
                if *next == ';' {
                    chars.next();
                    break;
                }
                entity.push(*next);
                chars.next();
            }

            let parsed = if is_hex {
                u32::from_str_radix(&entity, 16)
            } else {
                entity.parse::<u32>()
            };

            if let Ok(codepoint) = parsed {
                if let Some(decoded) = char::from_u32(codepoint) {
                    output.push(decoded);
                    continue;
                }
            }

            output.push_str("&#");
            if is_hex {
                output.push('x');
            }
            output.push_str(&entity);
            output.push(';');
        }

        output
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn Java_com_objectid_wallet_MainActivity_java_1init(
    mut env: JNIEnv,
    _class: JClass,
    context: JObject,
) {
    let context_for_current_verifier = env
        .new_local_ref(&context)
        .expect("Failed to create Android context local reference");

    rustls_platform_verifier::android::init_hosted(&mut env, context_for_current_verifier)
        .expect("Failed to initialize Android platform verifier");
    rustls_platform_verifier_05::android::init_hosted(&mut env, context)
        .expect("Failed to initialize Android platform verifier 0.5");
}
