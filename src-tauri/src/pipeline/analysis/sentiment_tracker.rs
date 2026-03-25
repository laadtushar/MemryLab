use crate::domain::ports::llm_provider::{ILlmProvider, LlmParams};
use crate::prompts::templates::{render_template, SENTIMENT_V1};

/// Sentiment score: -2 (very negative) to +2 (very positive).
#[derive(Debug, Clone)]
pub struct SentimentResult {
    pub chunk_id: String,
    pub score: f64,
    pub label: String,
}

/// Classify sentiment for a batch of chunks.
pub async fn classify_sentiment_batch(
    chunks: &[(String, String)], // (chunk_id, text)
    llm: &dyn ILlmProvider,
) -> Vec<SentimentResult> {
    let mut results = Vec::new();

    for (chunk_id, text) in chunks {
        let prompt = render_template(SENTIMENT_V1, &[("text", text.as_str())]);
        let params = LlmParams {
            temperature: Some(0.1),
            max_tokens: Some(20),
            ..Default::default()
        };

        match llm.complete(&prompt, &params).await {
            Ok(response) => {
                let (score, label) = parse_sentiment(&response);
                results.push(SentimentResult {
                    chunk_id: chunk_id.clone(),
                    score,
                    label,
                });
            }
            Err(e) => {
                log::warn!("Sentiment classification failed for {}: {}", chunk_id, e);
                results.push(SentimentResult {
                    chunk_id: chunk_id.clone(),
                    score: 0.0,
                    label: "neutral".to_string(),
                });
            }
        }
    }

    results
}

fn parse_sentiment(response: &str) -> (f64, String) {
    let cleaned = response.trim().to_lowercase();
    match cleaned.as_str() {
        s if s.contains("very_negative") || s.contains("very negative") => {
            (-2.0, "very_negative".to_string())
        }
        s if s.contains("negative") => (-1.0, "negative".to_string()),
        s if s.contains("very_positive") || s.contains("very positive") => {
            (2.0, "very_positive".to_string())
        }
        s if s.contains("positive") => (1.0, "positive".to_string()),
        _ => (0.0, "neutral".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sentiment() {
        assert_eq!(parse_sentiment("very_negative").0, -2.0);
        assert_eq!(parse_sentiment("negative").0, -1.0);
        assert_eq!(parse_sentiment("neutral").0, 0.0);
        assert_eq!(parse_sentiment("positive").0, 1.0);
        assert_eq!(parse_sentiment("very_positive").0, 2.0);
        assert_eq!(parse_sentiment("  Positive  ").0, 1.0);
        assert_eq!(parse_sentiment("something random").0, 0.0);
    }
}
