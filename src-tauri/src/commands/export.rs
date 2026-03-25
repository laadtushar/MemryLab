use tauri::State;

use crate::app_state::AppState;

/// Export all memory facts as a JSON string.
#[tauri::command]
pub fn export_memory_json(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let facts = state
        .memory_store
        .get_all(None, None)
        .map_err(|e| e.to_string())?;

    serde_json::to_string_pretty(&facts).map_err(|e| e.to_string())
}

/// Export all memory facts as Markdown.
#[tauri::command]
pub fn export_memory_markdown(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let facts = state
        .memory_store
        .get_all(None, None)
        .map_err(|e| e.to_string())?;

    let mut md = String::from("# Memory Palace — Exported Facts\n\n");
    md.push_str(&format!("*Exported: {}*\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")));
    md.push_str(&format!("Total facts: {}\n\n---\n\n", facts.len()));

    let mut by_category: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
    for fact in &facts {
        let cat = format!("{:?}", fact.category).to_lowercase();
        by_category.entry(cat).or_default().push(fact);
    }

    for (category, cat_facts) in &by_category {
        md.push_str(&format!("## {} ({})\n\n", capitalize(category), cat_facts.len()));
        for fact in cat_facts {
            let date = fact.first_seen.format("%Y-%m-%d");
            let conf = (fact.confidence * 100.0) as u32;
            md.push_str(&format!("- **[{}]** {} *(conf: {}%)*\n", date, fact.fact_text, conf));
        }
        md.push('\n');
    }

    Ok(md)
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
