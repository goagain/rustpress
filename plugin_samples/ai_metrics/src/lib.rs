use log::*;

mod logger;

wit_bindgen::generate!({
    world: "plugin-world",
    path: "./wit",
    pub_export_macro: true
});

use crate::{exports::rustpress::plugin::event_handler::*, rustpress::plugin::metrics::*};

struct AiMetricsPlugin;

impl Guest for AiMetricsPlugin {
    fn handle_filter(event: PluginFilterEvent) -> anyhow::Result<PluginFilterEvent, String> {
        logger::Logger::init();

        info!("Processing event for ai_metrics plugin");
        match event {
            _ => Ok(event),
        }
    }
    fn handle_action(event: PluginActionEvent) {
        logger::Logger::init();

        info!("Processing action for ai_metrics plugin");
        match event {
            PluginActionEvent::OnAiTokenUsed(data) => {
                on_ai_token_used(data);
            }
            PluginActionEvent::Unknown => {
                // Do nothing for unknown events
            }
            _ => (),
        }
    }
}
fn on_ai_token_used(data: OnAiTokenUsedData) {
    // Emit total tokens used as a counter
    let _ = crate::rustpress::plugin::metrics::emit(
        "ai_tokens_total",
        MetricType::Counter,
        &MetricValue::Counter(data.total_tokens as u64),
        &vec![
            ("plugin_id".to_string(), data.plugin_id.clone()),
            ("model".to_string(), data.model.clone()),
            ("operation".to_string(), data.operation.clone()),
        ],
    );

    // Emit prompt tokens as a counter
    let _ = crate::rustpress::plugin::metrics::emit(
        "ai_prompt_tokens_total",
        MetricType::Counter,
        &MetricValue::Counter(data.prompt_tokens as u64),
        &vec![
            ("plugin_id".to_string(), data.plugin_id.clone()),
            ("model".to_string(), data.model.clone()),
            ("operation".to_string(), data.operation.clone()),
        ],
    );

    // Emit completion tokens as a counter
    let _ = crate::rustpress::plugin::metrics::emit(
        "ai_completion_tokens_total",
        MetricType::Counter,
        &MetricValue::Counter(data.completion_tokens as u64),
        &vec![
            ("plugin_id".to_string(), data.plugin_id.clone()),
            ("model".to_string(), data.model.clone()),
            ("operation".to_string(), data.operation.clone()),
        ],
    );

    // Emit token usage ratio as a gauge (completion/prompt ratio)
    if data.prompt_tokens > 0 {
        let ratio = data.completion_tokens as f64 / data.prompt_tokens as f64;
        let _ = crate::rustpress::plugin::metrics::emit(
            "ai_token_ratio",
            MetricType::Gauge,
            &MetricValue::Gauge(ratio),
            &vec![
                ("plugin_id".to_string(), data.plugin_id.clone()),
                ("model".to_string(), data.model.clone()),
                ("operation".to_string(), data.operation.clone()),
            ],
        );
    }

    info!(
        "AI metrics emitted: plugin_id={}, model={}, operation={}, total_tokens={}, prompt_tokens={}, completion_tokens={}",
        data.plugin_id,
        data.model,
        data.operation,
        data.total_tokens,
        data.prompt_tokens,
        data.completion_tokens
    );
}

export!(AiMetricsPlugin);
