use regex::Regex;

pub struct PiiDetector {
    patterns: Vec<(String, Regex)>,
}

impl PiiDetector {
    pub fn new() -> Self {
        let patterns = vec![
            ("ssn", Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap()),
            ("credit_card", Regex::new(r"\b(?:\d{4}[- ]?){3}\d{4}\b").unwrap()),
            ("email", Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap()),
            ("phone_us", Regex::new(r"\b(?:\+?1[-.]?)?\(?\d{3}\)?[-.]?\d{3}[-.]?\d{4}\b").unwrap()),
            ("phone_intl", Regex::new(r"\b\+\d{1,3}[-.\s]?\d{4,14}\b").unwrap()),
            ("date_of_birth", Regex::new(r"(?i)\b(?:DOB|Date of Birth|born|birthday)[:\s]+\d{1,2}[/.-]\d{1,2}[/.-]\d{2,4}\b").unwrap()),
            ("ip_address", Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").unwrap()),
        ];
        Self {
            patterns: patterns.into_iter().map(|(n, r)| (n.to_string(), r)).collect(),
        }
    }

    /// Scan text and return list of PII types found.
    pub fn scan(&self, text: &str) -> Vec<String> {
        let mut found = Vec::new();
        for (name, pattern) in &self.patterns {
            if pattern.is_match(text) {
                found.push(name.clone());
            }
        }
        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_email() {
        let detector = PiiDetector::new();
        let results = detector.scan("Contact me at john@example.com for details.");
        assert!(results.contains(&"email".to_string()));
    }

    #[test]
    fn test_detect_ssn() {
        let detector = PiiDetector::new();
        let results = detector.scan("My SSN is 123-45-6789.");
        assert!(results.contains(&"ssn".to_string()));
    }

    #[test]
    fn test_no_pii() {
        let detector = PiiDetector::new();
        let results = detector.scan("I enjoy hiking in the mountains.");
        assert!(results.is_empty());
    }
}
