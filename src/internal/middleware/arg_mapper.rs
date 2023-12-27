use serde::{de::DeserializeOwned, Serialize};
use specta::Type;

// TODO: Should have `ArgumentMapperAdvanced` w/ `type = Output` for chaining these

/// TODO
#[deprecated]
pub trait ArgumentMapper: Send + Sync + 'static {
    /// TODO
    type State: Send + Sync + 'static;

    /// TODO
    type Input<T>: DeserializeOwned + Type + 'static
    where
        T: DeserializeOwned + Type + 'static;

    /// TODO
    fn map<T: Serialize + DeserializeOwned + Type + 'static>(
        arg: Self::Input<T>,
    ) -> (T, Self::State);
}

/// TODO
#[deprecated]
pub enum ArgumentMapperPassthrough {}

impl ArgumentMapper for ArgumentMapperPassthrough {
    type State = ();
    type Input<T> = T
    where
        T: DeserializeOwned + Type + 'static;

    fn map<T: Serialize + DeserializeOwned + Type + 'static>(
        arg: Self::Input<T>,
    ) -> (T, Self::State) {
        (arg, ())
    }
}
