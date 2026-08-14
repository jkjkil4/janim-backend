use std::fs;
use std::path::Path;

use serde_yaml::{Mapping, Value};

const DEFAULT_LANG: &str = "en";

fn merge_mapping(dst: &mut Mapping, src: &Mapping) {
    for (k, v) in src {
        match (dst.get_mut(k), v) {
            (Some(Value::Mapping(dst_map)), Value::Mapping(src_map)) => {
                merge_mapping(dst_map, src_map);
            }
            _ => {
                dst.insert(k.clone(), v.clone());
            }
        }
    }
}

fn extract_translations(todo: &Mapping) -> (Mapping, i32) {
    let mut extracted = Mapping::new();
    let mut moved_count = 0;

    for (key, value) in todo {
        if key.as_str() == Some("_version") {
            continue;
        }
        let text = key.as_str().unwrap();
        let lang_map = value.as_mapping().unwrap();

        let mut translations = Mapping::new();

        for (lang, translated) in lang_map {
            let translated_text = translated.as_str().unwrap();

            if lang == DEFAULT_LANG || translated_text != text {
                translations.insert(lang.clone(), translated.clone());
            }
        }

        if !translations.is_empty() {
            extracted.insert(key.clone(), Value::Mapping(translations));
            moved_count += 1;
        }
    }

    (extracted, moved_count)
}

fn remove_entries(todo: &mut Mapping, translations: &Mapping) {
    for (key, value) in translations {
        let src_lang_map = todo.get_mut(key).unwrap().as_mapping_mut().unwrap();
        let lang_map = value.as_mapping().unwrap();
        for lang in lang_map.keys() {
            src_lang_map.remove(lang);
        }
        if src_lang_map.is_empty() {
            todo.remove(key);
        }
    }
}

fn main() -> anyhow::Result<()> {
    let todo_path = Path::new("locales/TODO.yml");
    let app_path = Path::new("locales/app.yml");

    let todo_text = fs::read_to_string(todo_path)?;
    let mut todo: Mapping = serde_yaml::from_str(&todo_text)?;

    let mut app: Mapping = if app_path.exists() {
        let text = fs::read_to_string(app_path)?;
        serde_yaml::from_str(&text)?
    } else {
        let mut mapping = Mapping::new();
        mapping.insert("_version".into(), 2.into());
        mapping
    };

    let (translations, moved_count) = extract_translations(&todo);

    if translations.is_empty() {
        println!("No translations to merge");
        return Ok(());
    }

    // 合并到 app.yml
    merge_mapping(&mut app, &translations);

    let app_output = serde_yaml::to_string(&app)?;
    fs::write(app_path, app_output)?;

    // 从 TODO 删除已转移项
    remove_entries(&mut todo, &translations);

    if todo.len() <= 1 && todo.contains_key("_version") {
        // 只剩版本号时，无需 TODO 文件
        fs::remove_file(todo_path)?;
        println!("Removed empty TODO.yml");
    } else {
        let todo_output = serde_yaml::to_string(&todo)?;
        fs::write(todo_path, todo_output)?;
        println!("Updated TODO.yml");
    }

    println!("Moved {} entries", moved_count);

    Ok(())
}
