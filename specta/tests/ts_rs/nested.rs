use std::{sync::Arc, cell::Cell, rc::Rc};

use specta::{Type, ts_definition};

#[derive(Type)]
struct A {
    x1: Arc<i32>,
    y1: Cell<i32>
}

#[derive(Type)]
struct B {
    a1: Box<A>,
    #[specta(inline)]
    a2: A
}

#[derive(Type)]
struct C {
    b1: Rc<B>,
    #[specta(inline)]
    b2: B,
}

#[test]
fn test_nested() {
    assert_eq!(
        ts_definition::<C>(),
        "{ b1: B, b2: { a1: A, a2: { x1: number, y1: number } } }"
    );
}