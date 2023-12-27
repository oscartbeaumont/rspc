use std::pin::Pin;

pub trait Format {
    type Result; // TODO: Should we keep result???
    type Serializer: TODOSerializer;

    fn serializer(&self) -> Self::Serializer;

    fn into_result(ser: &mut Self::Serializer) -> Option<Self::Result>;
}

// TODO: Rename
pub trait TODOSerializer: Send {
    fn serialize_str(self: Pin<&mut Self>, s: &str);

    // TODO: Finish this
}
