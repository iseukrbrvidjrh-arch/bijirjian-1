use crate::domain::{ModelSource, ProviderModelInfo, ProviderType};

pub fn is_excluded_model_id(id: &str) -> bool {
    let lower = id.to_ascii_lowercase();
    const EXCLUDED: &[&str] = &[
        "embedding",
        "embed",
        "image",
        "imagen",
        "audio",
        "tts",
        "whisper",
        "moderation",
        "realtime",
        "transcri",
        "dall-e",
        "davinci",
        "babbage",
        "ada",
        "curie",
        "speech",
        "veo",
        "aqa",
        "nano-banana",
        "computer-use",
        "robotics",
    ];

    EXCLUDED.iter().any(|pattern| lower.contains(pattern))
}

pub fn to_provider_model_infos(
    provider_type: ProviderType,
    ids: Vec<String>,
    source: ModelSource,
) -> Vec<ProviderModelInfo> {
    ids.into_iter()
        .map(|id| ProviderModelInfo {
            label: crate::domain::model_label(provider_type, &id),
            provider_type,
            id,
            source,
        })
        .collect()
}

pub fn filter_deepseek_models(ids: Vec<String>) -> Vec<String> {
    let mut filtered: Vec<String> = ids
        .into_iter()
        .filter(|id| {
            let lower = id.to_ascii_lowercase();
            lower.starts_with("deepseek") && !is_excluded_model_id(id)
        })
        .collect();
    sort_deepseek_models(&mut filtered);
    filtered
}

pub fn filter_qwen_models(ids: Vec<String>) -> Vec<String> {
    let mut filtered: Vec<String> = ids
        .into_iter()
        .filter(|id| {
            let lower = id.to_ascii_lowercase();
            (lower.starts_with("qwen") || lower.contains("qwen")) && !is_excluded_model_id(id)
        })
        .collect();
    sort_qwen_models(&mut filtered);
    filtered
}

pub fn filter_openai_models(ids: Vec<String>) -> Vec<String> {
    let mut filtered: Vec<String> = ids
        .into_iter()
        .filter(|id| {
            let lower = id.to_ascii_lowercase();
            let looks_like_chat = lower.starts_with("gpt-")
                || lower.starts_with("o1")
                || lower.starts_with("o3")
                || lower.starts_with("o4")
                || lower.starts_with("chatgpt");
            looks_like_chat && !is_excluded_model_id(id)
        })
        .collect();
    sort_openai_models(&mut filtered);
    filtered
}

pub fn filter_gemini_models(ids: Vec<String>) -> Vec<String> {
    let mut filtered: Vec<String> = ids
        .into_iter()
        .filter(|id| {
            let lower = id.to_ascii_lowercase();
            lower.starts_with("gemini") && !is_excluded_model_id(id)
        })
        .collect();
    sort_gemini_models(&mut filtered);
    filtered
}

fn sort_deepseek_models(models: &mut [String]) {
    models.sort_by(|left, right| {
        model_rank(
            left,
            &["deepseek-v4-pro", "deepseek-v4-flash", "deepseek-chat"],
        )
        .cmp(&model_rank(
            right,
            &["deepseek-v4-pro", "deepseek-v4-flash", "deepseek-chat"],
        ))
        .then_with(|| left.cmp(right))
    });
}

fn sort_qwen_models(models: &mut [String]) {
    models.sort_by(|left, right| {
        model_rank(
            left,
            &[
                "qwen-max",
                "qwen-plus",
                "qwen-turbo",
                "qwen-flash",
                "qwen-long",
                "qwen-coder-plus",
            ],
        )
        .cmp(&model_rank(
            right,
            &[
                "qwen-max",
                "qwen-plus",
                "qwen-turbo",
                "qwen-flash",
                "qwen-long",
                "qwen-coder-plus",
            ],
        ))
        .then_with(|| left.cmp(right))
    });
}

fn sort_openai_models(models: &mut [String]) {
    models.sort_by(|left, right| {
        model_rank(
            left,
            &[
                "gpt-4.1",
                "gpt-4o",
                "gpt-4.1-mini",
                "gpt-4o-mini",
                "gpt-4-turbo",
                "o3-mini",
            ],
        )
        .cmp(&model_rank(
            right,
            &[
                "gpt-4.1",
                "gpt-4o",
                "gpt-4.1-mini",
                "gpt-4o-mini",
                "gpt-4-turbo",
                "o3-mini",
            ],
        ))
        .then_with(|| left.cmp(right))
    });
}

fn sort_gemini_models(models: &mut [String]) {
    models.sort_by(|left, right| {
        model_rank(
            left,
            &[
                "gemini-2.5-pro",
                "gemini-2.5-flash",
                "gemini-2.0-flash",
                "gemini-1.5-pro",
                "gemini-1.5-flash",
            ],
        )
        .cmp(&model_rank(
            right,
            &[
                "gemini-2.5-pro",
                "gemini-2.5-flash",
                "gemini-2.0-flash",
                "gemini-1.5-pro",
                "gemini-1.5-flash",
            ],
        ))
        .then_with(|| left.cmp(right))
    });
}

fn model_rank(model_id: &str, preferred: &[&str]) -> usize {
    preferred
        .iter()
        .position(|candidate| model_id.eq_ignore_ascii_case(candidate))
        .unwrap_or(preferred.len())
}

#[cfg(test)]
mod tests {
    use super::{
        filter_gemini_models, filter_openai_models, filter_qwen_models, is_excluded_model_id,
    };

    #[test]
    fn excludes_embedding_and_image_models() {
        assert!(is_excluded_model_id("text-embedding-3-small"));
        assert!(is_excluded_model_id("dall-e-3"));
        assert!(!is_excluded_model_id("gpt-4o"));
    }

    #[test]
    fn filters_openai_chat_models_and_sorts_newer_first() {
        let models = filter_openai_models(vec![
            "gpt-4o-mini".to_owned(),
            "text-embedding-3-small".to_owned(),
            "gpt-4.1".to_owned(),
            "gpt-4o".to_owned(),
        ]);

        assert_eq!(
            models,
            vec![
                "gpt-4.1".to_owned(),
                "gpt-4o".to_owned(),
                "gpt-4o-mini".to_owned()
            ]
        );
    }

    #[test]
    fn filters_qwen_models() {
        let models = filter_qwen_models(vec![
            "qwen-turbo".to_owned(),
            "qwen-plus".to_owned(),
            "whisper-1".to_owned(),
        ]);

        assert_eq!(
            models,
            vec!["qwen-plus".to_owned(), "qwen-turbo".to_owned()]
        );
    }

    #[test]
    fn filters_gemini_text_models() {
        let models = filter_gemini_models(vec![
            "gemini-2.0-flash".to_owned(),
            "gemini-embedding-001".to_owned(),
            "gemini-2.5-flash".to_owned(),
        ]);

        assert_eq!(
            models,
            vec!["gemini-2.5-flash".to_owned(), "gemini-2.0-flash".to_owned()]
        );
    }
}
