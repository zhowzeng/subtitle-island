use serde_json::{json, Value};
use std::time::Duration;

const OPENAI_RESPONSES_URL: &str = "https://api.openai.com/v1/responses";
const OPENAI_TRANSLATION_MODEL: &str = "gpt-6-luna";
const TRANSLATION_TIMEOUT: Duration = Duration::from_secs(10);
const TRANSLATION_INSTRUCTIONS: &str = "Translate the user's live subtitle text into Traditional Chinese as used in Taiwan. Reply with only the translation, without quotes, notes, or explanations.";
const CHINESE_HAN_RATIO: f32 = 0.5;

pub(crate) fn translation_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(TRANSLATION_TIMEOUT)
        .build()
        .map_err(|error| format!("Could not prepare Translation client: {error}"))
}

pub(crate) fn needs_translation(source_language: &str, text: &str) -> bool {
    source_language != "zh" && text.chars().any(char::is_alphabetic)
}

pub(crate) async fn translate_to_traditional_chinese(
    client: &reqwest::Client,
    api_key: &str,
    text: &str,
) -> Result<String, String> {
    let response = client
        .post(OPENAI_RESPONSES_URL)
        .bearer_auth(api_key)
        .json(&json!({
            "model": OPENAI_TRANSLATION_MODEL,
            "instructions": TRANSLATION_INSTRUCTIONS,
            "input": text
        }))
        .send()
        .await
        .map_err(|error| format!("Could not reach the Translation API: {error}"))?;

    let status = response.status();
    let body: Value = response
        .json()
        .await
        .map_err(|error| format!("Could not read Translation response: {error}"))?;

    if !status.is_success() {
        let message = body
            .pointer("/error/message")
            .and_then(Value::as_str)
            .unwrap_or("unknown error");
        return Err(format!("Translation API returned {status}: {message}"));
    }

    response_output_text(&body).ok_or_else(|| "Translation response had no text.".to_string())
}

fn response_output_text(body: &Value) -> Option<String> {
    let text: String = body
        .get("output")?
        .as_array()?
        .iter()
        .filter(|item| item.get("type").and_then(Value::as_str) == Some("message"))
        .filter_map(|item| item.get("content").and_then(Value::as_array))
        .flatten()
        .filter(|content| content.get("type").and_then(Value::as_str) == Some("output_text"))
        .filter_map(|content| content.get("text").and_then(Value::as_str))
        .collect();
    let text = text.trim();

    (!text.is_empty()).then(|| text.to_string())
}

// Each Han character counts as one token and each run of other letters counts as
// one word, so code-switched Chinese like "我們用 API 來做 demo" stays Chinese.
pub(crate) fn is_chinese_text(text: &str) -> bool {
    let mut han = 0;
    let mut other_words = 0;
    let mut in_other_word = false;

    for character in text.chars() {
        if is_kana_or_hangul(character) {
            return false;
        }

        if is_han(character) {
            han += 1;
            in_other_word = false;
        } else if character.is_alphabetic() {
            if !in_other_word {
                other_words += 1;
            }
            in_other_word = true;
        } else {
            in_other_word = false;
        }
    }

    let tokens = han + other_words;
    tokens > 0 && han as f32 / tokens as f32 >= CHINESE_HAN_RATIO
}

fn is_han(character: char) -> bool {
    matches!(
        character,
        '\u{3400}'..='\u{4DBF}' | '\u{4E00}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}'
    )
}

fn is_kana_or_hangul(character: char) -> bool {
    matches!(
        character,
        '\u{3040}'..='\u{30FF}' | '\u{1100}'..='\u{11FF}' | '\u{3130}'..='\u{318F}' | '\u{AC00}'..='\u{D7AF}'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chinese_text_is_detected() {
        assert!(is_chinese_text("今天的會議先到這裡。"));
        assert!(is_chinese_text("我們用 API 來做 demo"));
    }

    #[test]
    fn non_chinese_text_is_not_detected_as_chinese() {
        assert!(!is_chinese_text("Let's wrap up the meeting."));
        assert!(!is_chinese_text("今日は会議です。"));
        assert!(!is_chinese_text("안녕하세요"));
    }

    #[test]
    fn chinese_or_empty_text_skips_translation() {
        assert!(!needs_translation("zh", "Let's wrap up."));
        assert!(!needs_translation("auto", "  "));
        assert!(!needs_translation("en", "..."));
        assert!(needs_translation("auto", "Let's wrap up."));
    }

    #[test]
    fn response_output_text_joins_message_text() {
        let body = json!({
            "output": [
                { "type": "reasoning", "summary": [] },
                {
                    "type": "message",
                    "content": [{ "type": "output_text", "text": " 今天先到這裡。 " }]
                }
            ]
        });

        assert_eq!(
            response_output_text(&body).as_deref(),
            Some("今天先到這裡。")
        );
    }
}
