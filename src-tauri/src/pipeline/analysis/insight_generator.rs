use chrono::Utc;
use uuid::Uuid;

use crate::domain::models::insight::{Insight, InsightType};
use crate::domain::models::memory::MemoryFact;
use crate::domain::models::theme::ThemeSnapshot;
use crate::domain::ports::llm_provider::{ILlmProvider, LlmParams};
use crate::error::AppError;
use crate::prompts::templates::{render_template, INSIGHT_GENERATION_V1};

/// Generate top-N insights from analysis results.
pub async fn generate_insights(
    themes: &[ThemeSnapshot],
    beliefs: &[MemoryFact],
    llm: &dyn ILlmProvider,
    count: usize,
) -> Result<Vec<Insight>, AppError> {
    if themes.is_empty() && beliefs.is_empty() {
        return Ok(Vec::new());
    }

    let themes_summary = themes
        .iter()
        .map(|t| {
            format!(
                "- {} ({} to {}): {} (intensity: {:.1})",
                t.theme_label,
                t.time_window_start.format("%Y-%m"),
                t.time_window_end.format("%Y-%m"),
                t.description.as_deref().unwrap_or(""),
                t.intensity_score
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let beliefs_summary = beliefs
        .iter()
        .take(30) // Limit to avoid token overflow
        .map(|f| format!("- [{}] {} (confidence: {:.0}%)", format!("{:?}", f.category).to_lowercase(), f.fact_text, f.confidence * 100.0))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = render_template(
        INSIGHT_GENERATION_V1,
        &[
            ("themes", &themes_summary),
            ("beliefs", &beliefs_summary),
            ("sentiment_summary", "Sentiment analysis not yet available."),
            ("count", &count.to_string()),
        ],
    );

    let params = LlmParams {
        temperature: Some(0.7),
        max_tokens: Some(3000),
        ..Default::default()
    };

    let response = llm.complete(&prompt, &params).await?;
    Ok(parse_insight_response(&response))
}

fn parse_insight_response(response: &str) -> Vec<Insight> {
    let json_str = extract_json_array(response);
    let parsed: Result<Vec<InsightEntry>, _> = serde_json::from_str(&json_str);

    match parsed {
        Ok(entries) => entries
            .into_iter()
            .map(|entry| {
                let insight_type = match entry.insight_type.to_lowercase().as_str() {
                    "theme_shift" => InsightType::ThemeShift,
                    "sentiment_change" => InsightType::SentimentChange,
                    "belief_contradiction" => InsightType::BeliefContradiction,
                    "new_pattern" => InsightType::NewPattern,
                    "milestone_detected" => InsightType::MilestoneDetected,
                    _ => InsightType::NewPattern,
                };

                Insight {
                    id: Uuid::new_v4().to_string(),
                    insight_type,
                    title: entry.title,
                    body: entry.body,
                    time_range_start: None,
                    time_range_end: None,
                    supporting_evidence: vec![],
                    generated_at: Utc::now(),
                    prompt_version: Some("v1".to_string()),
                }
            })
            .collect(),
        Err(e) => {
            log::warn!("Failed to parse insight response: {}", e);
            Vec::new()
        }
    }
}

#[derive(serde::Deserialize)]
struct InsightEntry {
    title: String,
    body: String,
    insight_type: String,
}

fn extract_json_array(text: &str) -> String {
    let text = text.trim();
    let text = if text.starts_with("```") {
        let inner = text.trim_start_matches("```json").trim_start_matches("```");
        inner.trim_end_matches("```").trim()
    } else {
        text
    };
    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            return text[start..=end].to_string();
        }
    }
    text.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_insight_response() {
        let response = r#"[
            {"title": "Career shift detected", "body": "You moved from coding to management talk.", "insight_type": "theme_shift"},
            {"title": "Morning routine evolved", "body": "Started mentioning meditation in 2024.", "insight_type": "new_pattern"}
        ]"#;
        let insights = parse_insight_response(response);
        assert_eq!(insights.len(), 2);
        assert_eq!(insights[0].insight_type, InsightType::ThemeShift);
        assert_eq!(insights[1].insight_type, InsightType::NewPattern);
    }
}
