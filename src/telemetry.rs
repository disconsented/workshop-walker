//! Tracing setup. Writes spans to stdout and, when the config turns it on,
//! sends them to an OTLP collector such as a local Jaeger.

use opentelemetry::{global, trace::TracerProvider as _};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    propagation::TraceContextPropagator,
    trace::{SdkTracerProvider, Tracer},
};
use snafu::{ResultExt, Snafu};
use tracing_subscriber::{
    EnvFilter, Layer, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::app_config::Telemetry;

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum TelemetryError {
    #[snafu(display("Failed to build the OTLP span exporter for {endpoint}"))]
    BuildExporter {
        source: opentelemetry_otlp::ExporterBuildError,
        endpoint: String,
    },
}

/// Holds the tracer provider so that [`Guard::shutdown`] can flush the spans
/// that the batch processor still holds.
pub struct Guard(Option<SdkTracerProvider>);

impl Guard {
    /// Sends the spans that are still in the queue and stops the exporter.
    /// Call this before the process ends, or Jaeger loses the last batch.
    pub fn shutdown(self) {
        if let Some(provider) = self.0
            && let Err(error) = provider.shutdown()
        {
            eprintln!("failed to shut down the tracer provider: {error}");
        }
    }
}

/// The env var that selects what goes to the collector. It keeps the same
/// syntax as `RUST_LOG`, because both go through [`EnvFilter`].
pub const FILTER_VAR: &str = "OTEL_FILTER";

/// What the collector gets when [`FILTER_VAR`] is not set. `info` keeps the
/// Salvo request spans, `debug` adds the handlers, services, repositories and
/// actors, and `ractor=off` drops the actor lifetime spans, which last as long
/// as the process and say nothing about the work.
const DEFAULT_FILTER: &str = "info,workshop_walker=debug,ractor=off";

/// Installs the global subscriber. Each layer has its own filter: `RUST_LOG`
/// selects what stdout gets, [`FILTER_VAR`] selects what the collector gets.
pub fn init(config: &Telemetry) -> Result<Guard, TelemetryError> {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_span_events(FmtSpan::CLOSE)
        .with_filter(filter_from(EnvFilter::DEFAULT_ENV, "info"));

    let provider = if config.enabled {
        Some(provider(config)?)
    } else {
        None
    };
    let otel_layer = provider.as_ref().map(|provider| {
        global::set_text_map_propagator(TraceContextPropagator::new());
        global::set_tracer_provider(provider.clone());
        layer(provider.tracer(config.service_name.to_string()))
            .with_filter(filter_from(FILTER_VAR, DEFAULT_FILTER))
    });

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(otel_layer)
        .init();

    Ok(Guard(provider))
}

/// A bad filter string must not stop the program, so report it and fall back.
fn filter_from(var: &str, fallback: &str) -> EnvFilter {
    let filter = match EnvFilter::try_from_env(var) {
        Ok(filter) => filter,
        Err(error) => {
            if std::env::var(var).is_ok() {
                eprintln!("{var} is not a valid filter, falling back to {fallback}: {error}");
            }
            EnvFilter::new(fallback)
        }
    };

    // ractor instruments the lifetime of every actor it starts. The pipeline
    // spawns one actor per workshop item, so that is a span per item that says
    // only how long the actor lived. A directive added last wins, so this holds
    // even when the env var asks for everything.
    match "ractor=off".parse() {
        Ok(directive) => filter.add_directive(directive),
        Err(error) => {
            eprintln!("could not silence ractor: {error}");
            filter
        }
    }
}

fn provider(config: &Telemetry) -> Result<SdkTracerProvider, TelemetryError> {
    let exporter = SpanExporter::builder()
        .with_http()
        .with_endpoint(config.endpoint.as_str())
        .build()
        .context(BuildExporterSnafu {
            endpoint: config.endpoint.as_str(),
        })?;

    Ok(SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
            Resource::builder()
                .with_service_name(config.service_name.to_string())
                .build(),
        )
        .build())
}

/// Split out because the layer type is long and names the subscriber it wraps.
fn layer<S>(tracer: Tracer) -> impl Layer<S>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    tracing_opentelemetry::layer().with_tracer(tracer)
}

#[cfg(test)]
mod test {
    use opentelemetry::trace::TracerProvider as _;
    use tracing::{Level, debug_span, info_span};
    use tracing_subscriber::{EnvFilter, Layer as _, layer::SubscriberExt};

    use super::{layer, provider};
    use crate::app_config::Telemetry;

    /// Needs a collector on the endpoint that [`Telemetry::default`] names.
    /// Run it with `cargo test --bin workshop-walker -- --ignored telemetry`,
    /// then look for the `workshop-walker` service in the Jaeger UI.
    #[ignore = "needs a local collector"]
    #[test]
    fn sends_an_instrumented_span_to_the_collector() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .try_init();

        let config = Telemetry {
            enabled: true,
            ..Telemetry::default()
        };
        let provider = provider(&config).unwrap();
        let subscriber = tracing_subscriber::registry()
            .with(layer(provider.tracer(config.service_name.to_string())));

        tracing::subscriber::with_default(subscriber, || {
            info_span!("round trip", test = true).in_scope(|| {});
        });

        provider.force_flush().unwrap();
        provider.shutdown().unwrap();
    }

    /// The stdout filter must not hide a span from the collector. Look for
    /// `only for the collector` in the Jaeger UI; the same run prints nothing.
    #[ignore = "needs a local collector"]
    #[test]
    fn keeps_the_two_filters_apart() {
        let config = Telemetry {
            enabled: true,
            ..Telemetry::default()
        };
        let provider = provider(&config).unwrap();
        let subscriber = tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_test_writer()
                    .with_filter(EnvFilter::new("off")),
            )
            .with(
                layer(provider.tracer(config.service_name.to_string()))
                    .with_filter(EnvFilter::new("debug")),
            );

        tracing::subscriber::with_default(subscriber, || {
            debug_span!("only for the collector").in_scope(|| {});
        });

        provider.force_flush().unwrap();
        provider.shutdown().unwrap();
    }
}
