use std::pin::Pin;

use rspc_core::{Format, TODOSerializer};
use serde_json::Value;

pub(crate) struct SerdeJsonFormat {}

impl Format for SerdeJsonFormat {
    type Result = serde_json::Value;
    type Serializer = SerdeJsonSerializer;

    fn serializer(&self) -> Self::Serializer {
        SerdeJsonSerializer(None)
    }

    // TODO: Finish this method
    fn into_result(ser: &mut Self::Serializer) -> Option<Self::Result> {
        println!("{:?}", ser.0);
        ser.0.take()
    }
}

pub(crate) struct SerdeJsonSerializer(Option<Value>);

impl TODOSerializer for SerdeJsonSerializer {
    fn serialize_str(mut self: Pin<&mut Self>, s: &str) {
        self.0 = Some(Value::String(s.into()));
    }
}
