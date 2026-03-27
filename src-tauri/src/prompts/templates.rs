/// All prompt templates for the analysis and query pipeline.
/// Templates use {{variable}} placeholders for context injection.

pub const THEME_EXTRACTION_V1: &str = r#"You are analyzing personal journal entries and notes to extract recurring themes.

Given the following text excerpts from {{time_window}} ({{chunk_count}} excerpts):

{{chunks}}

Identify the major themes present in these texts. For each theme, provide:
1. A short label (2-4 words)
2. A brief description (1-2 sentences)
3. An intensity score from 0.0 to 1.0 (how prominent this theme is in the texts)

Respond in JSON format:
[
  {"label": "...", "description": "...", "intensity_score": 0.0}
]

Return only the JSON array, no other text."#;

pub const SENTIMENT_V1: &str = r#"Classify the sentiment of the following text into one of these categories:
very_negative, negative, neutral, positive, very_positive

Text: "{{text}}"

Respond with ONLY the category name."#;

pub const BELIEF_EXTRACTION_V1: &str = r#"You are analyzing personal writings to extract beliefs, opinions, values, and self-descriptions.

From the following text excerpts:

{{chunks}}

Extract any statements that represent:
- Beliefs or opinions the author holds
- Personal values
- Self-descriptions or identity statements
- Strong preferences

For each extracted fact, provide:
1. The fact statement (paraphrased concisely)
2. A category: "belief", "preference", "fact", or "self_description"
3. A confidence score from 0.0 to 1.0

Respond in JSON format:
[
  {"fact_text": "...", "category": "...", "confidence": 0.0}
]

Return only the JSON array, no other text."#;

pub const ENTITY_EXTRACTION_V1: &str = r#"Extract named entities from the following text. Identify:
- People (names, nicknames)
- Places (cities, countries, locations)
- Organizations (companies, groups, teams)
- Concepts/Topics (recurring ideas, themes)

Text: "{{text}}"

For each entity, provide:
1. The entity name
2. The entity type: "person", "place", "organization", "concept"

Respond in JSON format:
[
  {"name": "...", "entity_type": "..."}
]

Return only the JSON array, no other text."#;

pub const INSIGHT_GENERATION_V1: &str = r#"You are a personal insight analyst. You have access to the following data about someone's personal evolution:

Themes over time:
{{themes}}

Key beliefs and facts:
{{beliefs}}

Sentiment trends:
{{sentiment_summary}}

Based on this data, generate the {{count}} most interesting, surprising, or meaningful observations about this person's journey. Focus on:
- Unexpected changes or shifts
- Contradictions between earlier and later beliefs
- Patterns the person might not have noticed
- Significant emotional or thematic turning points

For each insight, provide:
1. A short title (5-10 words)
2. A body explanation (2-4 sentences)
3. The type: "theme_shift", "sentiment_change", "belief_contradiction", "new_pattern", or "milestone_detected"

IMPORTANT: Only reference dates and time periods that appear in the data above. Do NOT reference the current date or any dates not found in the provided data.

Respond in JSON format:
[
  {"title": "...", "body": "...", "insight_type": "..."}
]

Return only the JSON array, no other text."#;

pub const QUERY_CLASSIFICATION_V1: &str = r#"Classify the following search query into one of these types:
- semantic: searching by meaning or concept
- keyword: looking for specific words or phrases
- temporal: asking about a time period
- entity: asking about a specific person, place, or topic
- evolution: asking how something changed over time

Query: "{{query}}"

Also extract any relevant parameters:
- entities: names of people, places, or topics mentioned
- time_range: any time references (dates, periods)
- keywords: specific terms to search for

Respond in JSON:
{"type": "...", "entities": [], "time_range": null, "keywords": []}

Return only the JSON, no other text."#;

pub const RAG_RESPONSE_V1: &str = r#"You are a personal memory assistant. Answer the user's question using ONLY the provided context from their personal documents.

Context (from the user's own writings):
{{context}}

Relevant long-term memories:
{{memories}}

User's question: {{query}}

Instructions:
- Answer based solely on the provided context
- If the context doesn't contain enough information, say so
- Reference specific time periods when relevant
- Be empathetic and thoughtful — this is deeply personal data
- Keep your response concise but insightful"#;

pub const EVOLUTION_DIFF_V1: &str = "You are analyzing how a person's thinking evolved between two time periods.\n\n\
Period A ({period_a_label}):\n{period_a_text}\n\n\
Period B ({period_b_label}):\n{period_b_text}\n\n\
Compare these two periods. Respond in JSON:\n\
{\"summary\": \"2-3 sentence comparison\", \"sentiment_a\": \"positive/neutral/negative\", \"sentiment_b\": \"positive/neutral/negative\", \"key_shift\": \"what changed most\", \"quote_a\": \"most representative quote from period A\", \"quote_b\": \"most representative quote from period B\"}";

pub const CONTRADICTION_CHECK_V1: &str = "Analyze these two beliefs/preferences from the same person. Are they contradictory?\n\n\
Belief A: \"{fact_a}\"\nBelief B: \"{fact_b}\"\n\n\
Respond in JSON: {\"is_contradiction\": true/false, \"explanation\": \"brief explanation\", \"severity\": \"minor/moderate/major\"}";

pub const NARRATIVE_GENERATION_V1: &str = "You are writing a reflective narrative about a person's evolution.\n\n\
Theme: {subject}\n\
Time period: {time_range}\n\n\
Key moments and quotes:\n{evidence}\n\n\
Write a 2-3 paragraph narrative that tells the story of how this person's relationship with \"{subject}\" evolved. \
Use second person (\"you\"). Be empathetic and insightful. Reference specific quotes.\n\n\
IMPORTANT: Only mention dates and time periods that appear in the evidence above. Do NOT reference the current date or any dates not found in the provided evidence.";

/// Simple template rendering: replace {{variable}} with values.
pub fn render_template(template: &str, vars: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_template() {
        let template = "Hello {{name}}, you have {{count}} items.";
        let result = render_template(template, &[("name", "Alice"), ("count", "5")]);
        assert_eq!(result, "Hello Alice, you have 5 items.");
    }

    #[test]
    fn test_all_templates_are_non_empty() {
        assert!(!THEME_EXTRACTION_V1.is_empty());
        assert!(!SENTIMENT_V1.is_empty());
        assert!(!BELIEF_EXTRACTION_V1.is_empty());
        assert!(!ENTITY_EXTRACTION_V1.is_empty());
        assert!(!INSIGHT_GENERATION_V1.is_empty());
        assert!(!QUERY_CLASSIFICATION_V1.is_empty());
        assert!(!RAG_RESPONSE_V1.is_empty());
        assert!(!EVOLUTION_DIFF_V1.is_empty());
        assert!(!CONTRADICTION_CHECK_V1.is_empty());
        assert!(!NARRATIVE_GENERATION_V1.is_empty());
    }
}
