use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const MODEL_ALIAS_KEY_OPUS: &str = "opus";
pub const MODEL_ALIAS_KEY_SONNET: &str = "sonnet";
pub const MODEL_ALIAS_KEY_HAIKU: &str = "haiku";

#[derive(Debug, Clone, Copy)]
pub struct ServerModelMeta {
    pub api_id: &'static str,
    pub input_price_per_mtok: f64,
    pub output_price_per_mtok: f64,
    pub cache_read_price_per_mtok: f64,
    pub cache_write_price_per_mtok: f64,
    pub context_window: u32,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone, Copy)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct ModelPricing {
    pub input_price_per_mtok: f64,
    pub output_price_per_mtok: f64,
    pub cache_read_price_per_mtok: f64,
    pub cache_write_price_per_mtok: f64,
}

#[derive(Serialize, Deserialize, TS, Debug, Clone)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct ModelMeta {
    pub alias: String,
    pub api_id: String,
    pub pricing: ModelPricing,
    pub context_window: u32,
}

const META: &[(&str, ServerModelMeta)] = &[
    (
        MODEL_ALIAS_KEY_OPUS,
        ServerModelMeta {
            api_id: "claude-opus-4-7",
            input_price_per_mtok: 15.0,
            output_price_per_mtok: 75.0,
            cache_read_price_per_mtok: 1.5,
            cache_write_price_per_mtok: 18.75,
            context_window: 200_000,
        },
    ),
    (
        MODEL_ALIAS_KEY_SONNET,
        ServerModelMeta {
            api_id: "claude-sonnet-4-6",
            input_price_per_mtok: 3.0,
            output_price_per_mtok: 15.0,
            cache_read_price_per_mtok: 0.3,
            cache_write_price_per_mtok: 3.75,
            context_window: 200_000,
        },
    ),
    (
        MODEL_ALIAS_KEY_HAIKU,
        ServerModelMeta {
            api_id: "claude-haiku-4-5-20251001",
            input_price_per_mtok: 1.0,
            output_price_per_mtok: 5.0,
            cache_read_price_per_mtok: 0.1,
            cache_write_price_per_mtok: 1.25,
            context_window: 200_000,
        },
    ),
];

pub fn pricing(alias: &str) -> Option<&'static ServerModelMeta> {
    META.iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(alias))
        .map(|(_, m)| m)
}

pub fn context_window(alias: &str) -> Option<u32> {
    pricing(alias).map(|m| m.context_window)
}

/// Resolve by alias key OR full Anthropic model id.
pub fn resolve(alias_or_id: &str) -> Option<&'static ServerModelMeta> {
    META.iter()
        .find(|(k, m)| k.eq_ignore_ascii_case(alias_or_id) || m.api_id == alias_or_id)
        .map(|(_, m)| m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pricing_by_alias() {
        assert!(pricing(MODEL_ALIAS_KEY_OPUS).is_some());
        assert!(pricing("OPUS").is_some());
        assert!(pricing("nonexistent").is_none());
    }

    #[test]
    fn resolve_by_api_id() {
        assert!(resolve("claude-sonnet-4-6").is_some());
    }
}
