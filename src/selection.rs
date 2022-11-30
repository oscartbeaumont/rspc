#[macro_export]
macro_rules! selection {
    ( $s:expr, { $($n:ident),+ } ) => {{
        #[allow(non_camel_case_types)]
        mod selection {
            #[derive(serde::Serialize)]
            pub struct Selection<$($n,)*> {
                $(pub $n: $n,)*
            }

            impl<$($n: $crate::internal::specta::Type + 'static,)*> $crate::internal::specta::Type for Selection<$($n,)*> {
                const NAME: &'static str = "Selection";

                fn inline(opts: $crate::internal::specta::DefOpts, _generics: &[$crate::internal::specta::DataType]) -> $crate::internal::specta::DataType {
                    $crate::internal::specta::DataType::Object($crate::internal::specta::ObjectType {
                        name: "Selection".to_string(),
                        tag: None,
                        generics: vec![],
                        fields: vec![$(
                            $crate::internal::specta::ObjectField {
                                name: stringify!($n).to_string(),
                                ty: <$n as $crate::internal::specta::Type>::reference(
                                    $crate::internal::specta::DefOpts {
                                        parent_inline: false,
                                        type_map: opts.type_map,
                                    },
                                    &[]
                                ),
                                optional: false,
                                flatten: false
                            }
                        ),*],
                        type_id: Some(std::any::TypeId::of::<Self>()),
                    })
                }
            }
        }
        use selection::Selection;
        #[allow(non_camel_case_types)]
        Selection { $($n: $s.$n,)* }
    }};
    ( $s:expr, [{ $($n:ident),+ }] ) => {{
        #[allow(non_camel_case_types)]
        mod selection {
            #[derive(serde::Serialize)]
            pub struct Selection<$($n,)*> {
                $(pub $n: $n,)*
            }

            impl<$($n: $crate::internal::specta::Type + 'static,)*> $crate::internal::specta::Type for Selection<$($n,)*> {
                const NAME: &'static str = "Selection";

                fn inline(opts: $crate::internal::specta::DefOpts, _generics: &[$crate::internal::specta::DataType]) -> $crate::internal::specta::DataType {
                    $crate::internal::specta::DataType::Object($crate::internal::specta::ObjectType {
                        name: "Selection".to_string(),
                        tag: None,
                        generics: vec![],
                        fields: vec![$(
                            $crate::internal::specta::ObjectField {
                                name: stringify!($n).to_string(),
                                ty: <$n as $crate::internal::specta::Type>::reference(
                                    $crate::internal::specta::DefOpts {
                                        parent_inline: false,
                                        type_map: opts.type_map,
                                    },
                                    &[]
                                ),
                                optional: false,
                                flatten: false
                            }
                        ),*],
                        type_id: Some(std::any::TypeId::of::<Self>())
                    })
                }
            }
        }
        use selection::Selection;
        #[allow(non_camel_case_types)]
        $s.into_iter().map(|v| Selection { $($n: v.$n,)* }).collect::<Vec<_>>()
    }};
}

#[cfg(test)]
mod tests {
    use specta::{ts::inline, Type};

    fn ts_export_ref<T: Type>(_t: &T) -> String {
        inline::<T>()
    }

    #[derive(Clone)]
    #[allow(dead_code)]
    struct User {
        pub id: i32,
        pub name: String,
        pub email: String,
        pub age: i32,
        pub password: String,
    }

    #[test]
    fn test_selection_macros() {
        let user = User {
            id: 1,
            name: "Monty Beaumont".into(),
            email: "monty@otbeaumont.me".into(),
            age: 7,
            password: "password123".into(),
        };

        let s1 = selection!(user.clone(), { name, age });
        assert_eq!(s1.name, "Monty Beaumont".to_string());
        assert_eq!(s1.age, 7);
        assert_eq!(ts_export_ref(&s1), "{ name: string, age: number }");

        let users = vec![user; 3];
        let s2 = selection!(users, [{ name, age }]);
        assert_eq!(s2[0].name, "Monty Beaumont".to_string());
        assert_eq!(s2[0].age, 7);
        assert_eq!(ts_export_ref(&s2), "Array<{ name: string, age: number }>");
    }
}
