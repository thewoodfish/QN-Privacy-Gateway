//! Deterministic normalization for JSON-RPC requests.

use crate::privacy_mode::PrivacyMode;
use serde_json::{Map, Value};

pub fn normalize_rpc_request(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            // Sort keys lexicographically for deterministic output.
            let mut entries: Vec<(String, Value)> = map
                .into_iter()
                .map(|(k, v)| (k, normalize_rpc_request(v)))
                .collect();

            entries.sort_by(|a, b| a.0.cmp(&b.0));

            let mut sorted = Map::new();
            for (k, v) in entries {
                sorted.insert(k, v);
            }

            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(normalize_rpc_request).collect()),
        other => other,
    }
}

pub fn normalize_for_mode(mode: PrivacyMode, value: Value) -> Value {
    // Apply baseline normalization and then mode-specific rules.
    let mut normalized = normalize_rpc_request(value);

    if matches!(mode, PrivacyMode::Strict | PrivacyMode::Balanced) {
        if let Value::Object(ref mut map) = normalized {
            // Strip client-specific variance in request IDs.
            map.remove("id");
            if map.contains_key("jsonrpc") {
                map.insert("jsonrpc".to_string(), Value::String("2.0".to_string()));
            }
        }
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_normalize_rpc_request_sorts_keys() {
        let input = json!({
            "params": [],
            "method": "getSlot",
            "jsonrpc": "2.0",
            "id": 1
        });

        let normalized = normalize_rpc_request(input);
        let keys: Vec<&String> = normalized.as_object().unwrap().keys().collect();

        // Keys should be sorted alphabetically
        assert_eq!(keys, vec!["id", "jsonrpc", "method", "params"]);
    }

    #[test]
    fn test_normalize_for_mode_strict_removes_id() {
        let input = json!({
            "jsonrpc": "2.0",
            "id": 12345,
            "method": "getSlot",
            "params": []
        });

        let normalized = normalize_for_mode(PrivacyMode::Strict, input);
        let obj = normalized.as_object().unwrap();

        // ID should be removed in strict mode
        assert!(!obj.contains_key("id"));
        assert_eq!(obj.get("method").unwrap(), "getSlot");
    }

    #[test]
    fn test_normalize_for_mode_balanced_removes_id() {
        let input = json!({
            "jsonrpc": "2.0",
            "id": 67890,
            "method": "getBalance",
            "params": ["address123"]
        });

        let normalized = normalize_for_mode(PrivacyMode::Balanced, input);
        let obj = normalized.as_object().unwrap();

        // ID should be removed in balanced mode
        assert!(!obj.contains_key("id"));
        assert_eq!(obj.get("method").unwrap(), "getBalance");
    }

    #[test]
    fn test_normalize_for_mode_dev_keeps_id() {
        let input = json!({
            "jsonrpc": "2.0",
            "id": 99999,
            "method": "getSlot",
            "params": []
        });

        let normalized = normalize_for_mode(PrivacyMode::Dev, input);
        let obj = normalized.as_object().unwrap();

        // ID should be kept in dev mode
        assert_eq!(obj.get("id").unwrap(), 99999);
    }

    #[test]
    fn test_normalize_nested_objects() {
        let input = json!({
            "method": "getAccountInfo",
            "params": [{
                "encoding": "jsonParsed",
                "commitment": "finalized"
            }]
        });

        let normalized = normalize_rpc_request(input);
        let params = normalized.get("params").unwrap().as_array().unwrap();
        let nested_obj = params[0].as_object().unwrap();
        let keys: Vec<&String> = nested_obj.keys().collect();

        // Nested object keys should also be sorted
        assert_eq!(keys, vec!["commitment", "encoding"]);
    }

    #[test]
    fn test_same_semantic_requests_produce_same_hash() {
        let request1 = json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "getSlot",
            "params": []
        });

        let request2 = json!({
            "params": [],
            "method": "getSlot",
            "id": 999,
            "jsonrpc": "2.0"
        });

        let normalized1 = normalize_for_mode(PrivacyMode::Strict, request1);
        let normalized2 = normalize_for_mode(PrivacyMode::Strict, request2);

        // After normalization, they should be identical
        assert_eq!(
            serde_json::to_string(&normalized1).unwrap(),
            serde_json::to_string(&normalized2).unwrap()
        );
    }
}
