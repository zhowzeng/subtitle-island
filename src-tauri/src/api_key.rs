use std::{env, fs, path::PathBuf};

pub(crate) fn load_openai_api_key() -> Result<String, String> {
    if let Ok(dotenv_path) = env::current_dir().map(root_dotenv_path_from) {
        if fs::metadata(&dotenv_path).is_ok() {
            dotenvy::from_path_override(&dotenv_path)
                .map_err(|error| format!("Could not load .env: {error}"))?;
        }
    }

    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| "Missing OpenAI API key. Set OPENAI_API_KEY.".to_string())?;

    if api_key.trim().is_empty() {
        return Err("Missing OpenAI API key. Set OPENAI_API_KEY.".to_string());
    }

    Ok(api_key)
}

pub(crate) fn root_dotenv_path_from(current_dir: PathBuf) -> PathBuf {
    for path in current_dir.ancestors() {
        if path.join("package.json").is_file() && path.join("src-tauri").is_dir() {
            return path.join(".env");
        }
    }

    current_dir.join(".env")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_dotenv_path_uses_project_root_from_src_tauri() {
        let src_tauri_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root_dotenv_path_from(src_tauri_dir.clone());

        assert_eq!(
            path,
            src_tauri_dir
                .parent()
                .expect("src-tauri should have a project root parent")
                .join(".env")
        );
    }
}
