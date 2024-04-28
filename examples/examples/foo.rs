use std::str::FromStr;

use tracing::Collect;
use tracing_subscriber::{
    fmt::{
        self,
        format::{Format, Full, Pretty},
        FormatEvent, FormatFields,
    }, registry::LookupSpan, reload, subscribe::CollectExt, util::SubscriberInitExt, EnvFilter, Subscribe
};

enum Either<L, R> {
    Left(L),
    Right(R),
}

type EitherFormat = Either<Format<Full>, Format<Pretty>>;

impl<C, N, L: FormatEvent<C, N>, R: FormatEvent<C, N>> FormatEvent<C, N> for Either<L, R>
where
    C: Collect + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &fmt::FmtContext<'_, C, N>,
        writer: fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        match self {
            Either::Left(left) => left.format_event(ctx, writer, event),
            Either::Right(right) => right.format_event(ctx, writer, event),
        }
    }
}

fn main() {
    let filter = EnvFilter::from_str("info").unwrap();
    let default_layer = tracing_subscriber::fmt::Subscriber::new()
        .with_ansi(true)
        .event_format(EitherFormat::Left(Format::default()))
        .with_filter(filter);
    let (default_layer, reload_handle) = reload::Subscriber::new(default_layer);
    tracing_subscriber::registry().with(default_layer).init();
    tracing::info!("Normal");
    reload_handle
        .modify(|layer| {
            *layer.inner_mut().event_format_mut() = EitherFormat::Right(Format::default().pretty());
        })
        .unwrap();
    tracing::info!("Pretty");
}
