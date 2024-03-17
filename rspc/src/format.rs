use serde::Serialize;

/// TODO
pub trait Format {
    type Output;

    fn new() -> Self;

    fn serialize<T: Serialize>(&mut self, v: T);

    fn take(self) -> Self::Output;
}

pub(crate) trait DynFormat {
    fn serialize(&mut self, v: &dyn erased_serde::Serialize);
}

impl<T: Format> DynFormat for T {
    fn serialize(&mut self, v: &dyn erased_serde::Serialize) {
        <T as Format>::serialize(self, v)
    }
}

// TODO: Move into another crate

/// TODO
pub struct JsonValue {
    result: serde_json::Value,
}

impl JsonValue {
    pub fn new() -> Self {
        Self {
            result: serde_json::Value::Null,
        }
    }
}

impl Format for JsonValue {
    type Output = serde_json::Value;

    fn new() -> Self {
        Self {
            result: serde_json::Value::Null,
        }
    }

    fn serialize<T: Serialize>(&mut self, v: T) {
        self.result = serde_json::to_value(v).unwrap(); // TODO: Error handling
    }

    fn take(self) -> Self::Output {
        self.result
    }
}

// TODO: Json into Vec<u8> buffer
// TODO: FormData
