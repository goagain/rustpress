//! Host-side implementation for the plugin metrics interface

use crate::plugin::rustpress::plugin::metrics::*;

#[async_trait::async_trait]
impl Host for super::super::PluginHostState {
    /// Emit a metric with the plugin ID as prefix
    /// The full metric name will be: "plugin_{plugin_id}_{name}"
    async fn emit(
        &mut self,
        name: String,
        metric_type: MetricType,
        value: MetricValue,
        labels: Vec<(String, String)>,
    ) -> Result<(), wasmtime::Error> {
        // Create the full metric name with plugin ID prefix
        let full_name = format!("plugin_{}_{}", self.plugin_id, name);

        // Convert labels to a more usable format
        let label_pairs: Vec<(&str, &str)> = labels
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        // Emit the metric based on its type
        match (metric_type, value) {
            (MetricType::Counter, MetricValue::Counter(count)) => {
                let counter = prometheus::register_counter_vec!(
                    &full_name,
                    "Plugin counter metric",
                    &label_pairs.iter().map(|(k, _)| *k).collect::<Vec<_>>()
                )
                .unwrap_or_else(|_| {
                    // If registration fails, try to get existing metric
                    prometheus::CounterVec::new(
                        prometheus::Opts::new(&full_name, "Plugin counter metric"),
                        &label_pairs.iter().map(|(k, _)| *k).collect::<Vec<_>>(),
                    )
                    .unwrap()
                });

                // Set the counter value
                if let Ok(counter) = counter.get_metric_with_label_values(
                    &label_pairs.iter().map(|(_, v)| *v).collect::<Vec<_>>(),
                ) {
                    counter.reset(); // Reset to 0 first
                    counter.inc_by(count as f64); // Then increment by the desired value
                }
            }

            (MetricType::Gauge, MetricValue::Gauge(gauge_value)) => {
                let gauge = prometheus::register_gauge_vec!(
                    &full_name,
                    "Plugin gauge metric",
                    &label_pairs.iter().map(|(k, _)| *k).collect::<Vec<_>>()
                )
                .unwrap_or_else(|_| {
                    prometheus::GaugeVec::new(
                        prometheus::Opts::new(&full_name, "Plugin gauge metric"),
                        &label_pairs.iter().map(|(k, _)| *k).collect::<Vec<_>>(),
                    )
                    .unwrap()
                });

                if let Ok(gauge) = gauge.get_metric_with_label_values(
                    &label_pairs.iter().map(|(_, v)| *v).collect::<Vec<_>>(),
                ) {
                    gauge.set(gauge_value);
                }
            }

            (MetricType::Histogram, MetricValue::Histogram(histogram_metric)) => {
                // For histogram, we need to create a custom histogram with the provided buckets
                let histogram = prometheus::register_histogram_vec!(
                    &full_name,
                    "Plugin histogram metric",
                    &label_pairs.iter().map(|(k, _)| *k).collect::<Vec<_>>()
                )
                .unwrap_or_else(|_| {
                    // Create histogram with custom buckets
                    let buckets: Vec<f64> = histogram_metric.buckets.iter().map(|b| b.le).collect();

                    prometheus::HistogramVec::new(
                        prometheus::HistogramOpts {
                            common_opts: prometheus::Opts::new(
                                &full_name,
                                "Plugin histogram metric",
                            ),
                            buckets,
                        },
                        &label_pairs.iter().map(|(k, _)| *k).collect::<Vec<_>>(),
                    )
                    .unwrap()
                });

                if let Ok(histogram) = histogram.get_metric_with_label_values(
                    &label_pairs.iter().map(|(_, v)| *v).collect::<Vec<_>>(),
                ) {
                    // For histogram, we need to simulate observations
                    // Since Prometheus histograms don't allow direct setting of values,
                    // we'll observe the sample_sum divided by sample_count as an approximation
                    if histogram_metric.sample_count > 0 {
                        let average =
                            histogram_metric.sample_sum / histogram_metric.sample_count as f64;
                        for _ in 0..histogram_metric.sample_count {
                            histogram.observe(average);
                        }
                    }
                }
            }

            // Mismatched metric type and value
            _ => {
                tracing::warn!(
                    "Plugin {} emitted metric {} with mismatched type {:?} and value type",
                    self.plugin_id,
                    full_name,
                    metric_type
                );
            }
        }

        Ok(())
    }
}
