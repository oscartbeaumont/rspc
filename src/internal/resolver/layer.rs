use serde::de::DeserializeOwned;
use serde_json::Value;
use specta::Type;
use std::marker::PhantomData;

#[cfg(feature = "tracing")]
mod tracing_only {
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };

    use super::*;

    use pin_project_lite::pin_project;

    pin_project! {
        // TODO: Try and remove
        pub struct Instrumented<S> {
            #[pin]
            pub(super) stream: S,
            pub(super) span: Option<tracing::Span>,
        }
    }

    impl<S: Body + Send + 'static> Body for Instrumented<S> {
        fn poll_next(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Option<Result<Value, ExecError>>> {
            let this = self.project();
            let _span = this.span.as_ref().map(|s| s.enter());
            this.stream.poll_next(cx)
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (0, None)
        }
    }
}

pub(crate) struct ResolverLayer<T, TArg> {
    func: T,
    phantom: PhantomData<fn() -> TArg>,
}

impl<T, TArg> ResolverLayer<T, TArg> {
    pub(crate) fn new(func: T) -> Self {
        Self {
            func,
            phantom: PhantomData,
        }
    }
}

// TODO: For `T: ResolverFunction` or something like that to simplify the generics
impl<T, TArg, TLayerCtx, S> SealedLayer<TLayerCtx> for ResolverLayer<T, TArg>
where
    TLayerCtx: Send + Sync + 'static,
    TArg: Type + DeserializeOwned + 'static,
    T: Fn(TLayerCtx, TArg, RequestContext) -> Result<S, ExecError> + Send + Sync + 'static,
    S: Body + Send + 'static,
{
    #[cfg(feature = "tracing")]
    type Stream<'a> = tracing_only::Instrumented<S>;

    #[cfg(not(feature = "tracing"))]
    type Stream<'a> = S;

    fn call(
        &self,
        ctx: TLayerCtx,
        input: Value,
        req: RequestContext,
    ) -> Result<Self::Stream<'_>, ExecError> {
        #[cfg(feature = "tracing")]
        let span = req.span();

        #[cfg(feature = "tracing")]
        let _enter = span.as_ref().map(|s| s.enter());

        // TODO: Using content types lol
        let input = serde_json::from_value(input).map_err(ExecError::DeserializingArgErr)?;
        let result = (self.func)(ctx, input, req);

        #[cfg(feature = "tracing")]
        drop(_enter);

        #[cfg(not(feature = "tracing"))]
        return result;

        #[cfg(feature = "tracing")]
        return Ok(tracing_only::Instrumented {
            stream: result?,
            span,
        });
    }
}
