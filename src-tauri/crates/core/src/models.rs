use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const MODEL_ALIAS_KEY_OPUS: &str = "opus";
pub const MODEL_ALIAS_KEY_SONNET: &str = "sonnet";
pub const MODEL_ALIAS_KEY_HAIKU: &str = "haiku";
pub const MODEL_ALIAS_KEY_FABLE: &str = "fable";

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

/// Published per-MTok rates. Input and output come from the pricing table;
/// cache read and cache write follow the standard 0.1x / 1.25x of input, with
/// one documented exception (Fable 5.1 reads at $0.25).
///
/// Ordered newest-first within a family so a substring fallback lands on the
/// current generation.
const META: &[(&str, ServerModelMeta)] = &[
    (
        MODEL_ALIAS_KEY_FABLE,
        ServerModelMeta {
            api_id: "claude-fable-5-1",
            input_price_per_mtok: 10.0,
            output_price_per_mtok: 50.0,
            cache_read_price_per_mtok: 0.25,
            cache_write_price_per_mtok: 12.5,
            context_window: 1_000_000,
        },
    ),
    (
        MODEL_ALIAS_KEY_FABLE,
        ServerModelMeta {
            api_id: "claude-fable-5",
            input_price_per_mtok: 10.0,
            output_price_per_mtok: 50.0,
            cache_read_price_per_mtok: 1.0,
            cache_write_price_per_mtok: 12.5,
            context_window: 1_000_000,
        },
    ),
    (
        MODEL_ALIAS_KEY_OPUS,
        ServerModelMeta {
            api_id: "claude-opus-5",
            input_price_per_mtok: 5.0,
            output_price_per_mtok: 25.0,
            cache_read_price_per_mtok: 0.5,
            cache_write_price_per_mtok: 6.25,
            context_window: 1_000_000,
        },
    ),
    (
        MODEL_ALIAS_KEY_OPUS,
        ServerModelMeta {
            api_id: "claude-opus-4-8",
            input_price_per_mtok: 5.0,
            output_price_per_mtok: 25.0,
            cache_read_price_per_mtok: 0.5,
            cache_write_price_per_mtok: 6.25,
            context_window: 1_000_000,
        },
    ),
    (
        MODEL_ALIAS_KEY_OPUS,
        ServerModelMeta {
            api_id: "claude-opus-4-7",
            input_price_per_mtok: 5.0,
            output_price_per_mtok: 25.0,
            cache_read_price_per_mtok: 0.5,
            cache_write_price_per_mtok: 6.25,
            context_window: 1_000_000,
        },
    ),
    (
        MODEL_ALIAS_KEY_SONNET,
        ServerModelMeta {
            api_id: "claude-sonnet-5",
            input_price_per_mtok: 2.0,
            output_price_per_mtok: 10.0,
            cache_read_price_per_mtok: 0.2,
            cache_write_price_per_mtok: 2.5,
            context_window: 1_000_000,
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
            context_window: 1_000_000,
        },
    ),
    (
        MODEL_ALIAS_KEY_HAIKU,
        ServerModelMeta {
            api_id: "claude-haiku-4-5",
            input_price_per_mtok: 1.0,
            output_price_per_mtok: 5.0,
            cache_read_price_per_mtok: 0.1,
            cache_write_price_per_mtok: 1.25,
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

/// Resolve for *pricing historical data*, where the model id is whatever the
/// CLI recorded at the time — `claude-3-5-sonnet-20241022`,
/// `claude-sonnet-4-5-20250929`, and so on. Exact ids in `META` win; anything
/// else falls back to the family named in the id.
///
/// The fallback prices an old model at today's rate for its family, which is
/// approximate. That beats the alternative: an unmatched id silently costs
/// zero, so a year of history reads as free. Callers should report which ids
/// fell through so the number can be qualified.
pub fn resolve_for_pricing(model_id: &str) -> Option<&'static ServerModelMeta> {
    if let Some(exact) = resolve(model_id) {
        return Some(exact);
    }
    let lower = model_id.to_ascii_lowercase();
    for family in [
        MODEL_ALIAS_KEY_FABLE,
        MODEL_ALIAS_KEY_OPUS,
        MODEL_ALIAS_KEY_SONNET,
        MODEL_ALIAS_KEY_HAIKU,
    ] {
        if lower.contains(family) {
            return pricing(family);
        }
    }
    None
}

/// Cost in USD for one turn's token counts, at `model`'s rates.
pub fn cost_usd(model_id: &str, input: u64, output: u64, cache_read: u64, cache_write: u64) -> Option<f64> {
    let m = resolve_for_pricing(model_id)?;
    let per_mtok = |tokens: u64, price: f64| (tokens as f64) * price / 1_000_000.0;
    Some(
        per_mtok(input, m.input_price_per_mtok)
            + per_mtok(output, m.output_price_per_mtok)
            + per_mtok(cache_read, m.cache_read_price_per_mtok)
            + per_mtok(cache_write, m.cache_write_price_per_mtok),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pricing_by_alias() {
        assert_eq!(pricing(MODEL_ALIAS_KEY_OPUS).unwrap().api_id, "claude-opus-5");
        assert_eq!(pricing(MODEL_ALIAS_KEY_FABLE).unwrap().api_id, "claude-fable-5-1");
        assert!(pricing("OPUS").is_some());
        assert!(pricing("nonexistent").is_none());
    }

    #[test]
    fn resolve_by_api_id() {
        assert!(resolve("claude-sonnet-4-6").is_some());
    }

    #[test]
    fn pricing_falls_back_to_the_family_in_the_id() {
        // Historical ids that are not in META must not price at zero.
        assert_eq!(
            resolve_for_pricing("claude-3-5-sonnet-20241022").unwrap().api_id,
            "claude-sonnet-5"
        );
        assert_eq!(
            resolve_for_pricing("claude-opus-4-1-20250805").unwrap().api_id,
            "claude-opus-5"
        );
        // Exact ids still win over the family fallback.
        assert_eq!(
            resolve_for_pricing("claude-opus-4-7").unwrap().api_id,
            "claude-opus-4-7"
        );
        assert_eq!(
            resolve_for_pricing("claude-fable-5-1").unwrap().input_price_per_mtok,
            10.0
        );
        // An id naming no known family stays unpriced, so callers can report it.
        assert!(resolve_for_pricing("gpt-4o").is_none());
    }

    #[test]
    fn cost_matches_a_hand_computed_value() {
        // 1M input + 1M output at sonnet rates: $3 + $15.
        let c = cost_usd("claude-sonnet-4-6", 1_000_000, 1_000_000, 0, 0).unwrap();
        assert!((c - 18.0).abs() < 1e-9, "got {c}");
        // Cache reads are an order of magnitude cheaper than fresh input.
        let cached = cost_usd("claude-sonnet-4-6", 0, 0, 1_000_000, 0).unwrap();
        assert!((cached - 0.3).abs() < 1e-9, "got {cached}");
        // Opus 5: $5 in + $25 out per MTok.
        let opus = cost_usd("claude-opus-5", 1_000_000, 1_000_000, 0, 0).unwrap();
        assert!((opus - 30.0).abs() < 1e-9, "got {opus}");
        assert!(cost_usd("gpt-4o", 1, 1, 1, 1).is_none());
    }
}
