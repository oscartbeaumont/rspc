use std::{marker::PhantomData, pin::Pin};

use crate::TODOSerializer;

pub struct Serializer<'a> {
    serializer: Pin<&'a mut dyn TODOSerializer>,
    phantom: PhantomData<&'a ()>,
}

impl<'a> Serializer<'a> {
    pub(crate) fn new(serializer: Pin<&'a mut dyn TODOSerializer>) -> Self {
        Self {
            serializer,
            phantom: PhantomData,
        }
    }

    // TODO: Should this be `async` so we can yield and reset the serializer state for multiple values???
    pub fn serialize<T: serde::Serialize>(self, value: &T) {
        // TODO: Properly hook this up with Serde

        self.serializer.serialize_str("Hello World");
    }

    // TODO: API for bytes
}
