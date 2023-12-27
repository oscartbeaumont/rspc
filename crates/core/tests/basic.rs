use std::{pin::Pin, sync::Arc};

use futures::StreamExt;
use rspc_core::{Executor, Format, Procedure, TODOSerializer};
use serde_json::Value;

#[tokio::test]
async fn test_procedure() {
    let _p: Procedure = Arc::new(|ctx| {
        Box::pin(async move {
            // let _input: String = erased_serde::deserialize(ctx.arg.unwrap()).unwrap();

            // Serialize a single value
            ctx.result.serialize(&"todo");
        })
    });

    let _p: Procedure = Arc::new(|ctx| {
        Box::pin(async move {
            // let _input: String = erased_serde::deserialize(ctx.arg.unwrap()).unwrap();
            // ctx.result.erased_serialize_str("todo");

            // Serialize multiple values
            // TODO
            // for i in 0..5 {
            //     ctx.result.serialize(&i);
            // }
        })
    });

    let _p: Procedure = Arc::new(|ctx| {
        Box::pin(async move {
            // let _input: String = erased_serde::deserialize(ctx.arg.unwrap()).unwrap();

            // TODO: Stream entire file worth of bytes
        })
    });
}

#[tokio::test]
async fn test_executor() {
    let mut executor = Executor::default();

    executor.insert(
        "demo".into(),
        Arc::new(|ctx| {
            Box::pin(async move {
                // let _input: String = erased_serde::deserialize(ctx.arg.unwrap()).unwrap();

                // Serialize a single value
                ctx.result.serialize(&"todo");
            })
        }),
    );

    let executor = Arc::new(executor);
    let format = Arc::new(SerdeJsonFormat {});

    let result = executor.execute("demo", format).collect::<Vec<_>>().await;
    println!("{:?}", result);
    panic!("done");
}

// TODO: Assert a task can be queued onto an async runtime like Tokio

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
