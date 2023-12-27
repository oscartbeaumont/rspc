// pin_project_lite::pin_project! {
//     #[project = _PinnedOptionProj]
pub enum PinnedOption<T> {
    Some {
        // #[pin]
        v: T,
    },
    None,
}
// }

impl<T> From<T> for PinnedOption<T> {
    fn from(value: T) -> Self {
        Self::Some { v: value }
    }
}

// TODO: Avoid need for manually expanding the `pin_project` macro
/* The `cargo expand` output for `pin_project!` so that we can make `PinnedOptionProj` public */
#[doc(hidden)]
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
#[allow(clippy::unknown_lints)]
#[allow(clippy::mut_mut)]
#[allow(clippy::redundant_pub_crate)]
#[allow(clippy::ref_option_ref)]
#[allow(clippy::type_repetition_in_bounds)]
pub enum PinnedOptionProj<'__pin, T>
where
    PinnedOption<T>: '__pin,
{
    Some {
        v: ::pin_project_lite::__private::Pin<&'__pin mut T>,
    },
    None,
}
#[allow(single_use_lifetimes)]
#[allow(clippy::unknown_lints)]
#[allow(clippy::used_underscore_binding)]
#[allow(unsafe_code)] // <- Custom
#[allow(warnings)] // <- Custom
const _: () = {
    impl<T> PinnedOption<T> {
        #[doc(hidden)]
        #[inline]
        pub fn project<'__pin>(
            self: ::pin_project_lite::__private::Pin<&'__pin mut Self>,
        ) -> PinnedOptionProj<'__pin, T> {
            unsafe {
                match self.get_unchecked_mut() {
                    Self::Some { v } => PinnedOptionProj::Some {
                        v: ::pin_project_lite::__private::Pin::new_unchecked(v),
                    },
                    Self::None => PinnedOptionProj::None,
                }
            }
        }
    }
    #[allow(non_snake_case)]
    pub struct __Origin<'__pin, T> {
        __dummy_lifetime: ::pin_project_lite::__private::PhantomData<&'__pin ()>,
        Some: (T),
        None: (),
    }
    impl<'__pin, T> ::pin_project_lite::__private::Unpin for PinnedOption<T> where
        __Origin<'__pin, T>: ::pin_project_lite::__private::Unpin
    {
    }
    trait MustNotImplDrop {}
    #[allow(clippy::drop_bounds, drop_bounds)]
    impl<T: ::pin_project_lite::__private::Drop> MustNotImplDrop for T {}
    impl<T> MustNotImplDrop for PinnedOption<T> {}
};
