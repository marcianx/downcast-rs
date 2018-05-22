//! Rust enums are great for types where all variations are known beforehand. But in
//! the case where you want to implement a container of user-defined types, an
//! open-ended type like a trait object is needed. In some cases, it is useful to
//! cast the trait object back into its original concrete type to access additional
//! functionality and performant inlined implementations.
//!
//! `downcast-rs` adds basic downcasting support to trait objects, supporting **type
//! parameters**, **associated types**, and **constraints**.
//!
//! To make a trait downcastable, make it extend the `downcast::Downcast` trait and
//! invoke `impl_downcast!` on it as follows:
//!
//! ```
//! # #[macro_use]
//! # extern crate downcast_rs;
//! # use downcast_rs::Downcast;
//! trait Trait: Downcast {}
//! impl_downcast!(Trait);
//!
//! // With type parameters.
//! trait TraitGeneric1<T>: Downcast {}
//! impl_downcast!(TraitGeneric1<T>);
//!
//! // With associated types.
//! trait TraitGeneric2: Downcast { type G; type H; }
//! impl_downcast!(TraitGeneric2 assoc G, H);
//!
//! // With constraints on types.
//! trait TraitGeneric3<T: Copy>: Downcast {
//!     type H: Clone;
//! }
//! impl_downcast!(TraitGeneric3<T> assoc H where T: Copy, H: Clone);
//!
//! // With concrete types.
//! trait TraitConcrete1<T: Copy>: Downcast {}
//! impl_downcast!(concrete TraitConcrete1<u32>);
//!
//! trait TraitConcrete2<T: Copy>: Downcast { type H; }
//! impl_downcast!(concrete TraitConcrete2<u32> assoc H=f64);
//! # fn main() {}
//! ```
//!
//! # Example without generics
//!
//! ```
//! #[macro_use]
//! extern crate downcast_rs;
//! use downcast_rs::Downcast;
//!
//! // To create a trait with downcasting methods, extend `Downcast` and run
//! // impl_downcast!() on the trait.
//! trait Base: Downcast {}
//! impl_downcast!(Base);
//!
//! // Concrete types implementing Base.
//! #[derive(Debug)]
//! struct Foo(u32);
//! impl Base for Foo {}
//! #[derive(Debug)]
//! struct Bar(f64);
//! impl Base for Bar {}
//!
//! fn main() {
//!     // Create a trait object.
//!     let mut base: Box<Base> = Box::new(Foo(42));
//!
//!     // Try sequential downcasts.
//!     if let Some(foo) = base.downcast_ref::<Foo>() {
//!         assert_eq!(foo.0, 42);
//!     } else if let Some(bar) = base.downcast_ref::<Bar>() {
//!         assert_eq!(bar.0, 42.0);
//!     }
//!
//!     assert!(base.is::<Foo>());
//!
//!     // Fail to convert Box<Base> into Box<Bar>.
//!     let res = base.downcast::<Bar>();
//!     assert!(res.is_err());
//!     let base = res.unwrap_err();
//!     // Convert Box<Base> into Box<Foo>.
//!     assert_eq!(42, base.downcast::<Foo>().map_err(|_| "Shouldn't happen.").unwrap().0);
//! }
//! ```
//!
//! # Example with a generic trait with associated types and constraints
//!
//! ```
//! #[macro_use]
//! extern crate downcast_rs;
//! use downcast_rs::Downcast;
//!
//! // To create a trait with downcasting methods, extend `Downcast` and run
//! // impl_downcast!() on the trait.
//! trait Base<T: Clone>: Downcast { type H: Copy; }
//! impl_downcast!(Base<T> assoc H where T: Clone, H: Copy);
//! // or: impl_downcast!(concrete Base<u32> assoc H=f32)
//!
//! // Concrete types implementing Base.
//! struct Foo(u32);
//! impl Base<u32> for Foo { type H = f32; }
//! struct Bar(f64);
//! impl Base<u32> for Bar { type H = f32; }
//!
//! fn main() {
//!     // Create a trait object.
//!     let mut base: Box<Base<u32, H=f32>> = Box::new(Bar(42.0));
//!
//!     // Try sequential downcasts.
//!     if let Some(foo) = base.downcast_ref::<Foo>() {
//!         assert_eq!(foo.0, 42);
//!     } else if let Some(bar) = base.downcast_ref::<Bar>() {
//!         assert_eq!(bar.0, 42.0);
//!     }
//!
//!     assert!(base.is::<Bar>());
//! }
//! ```

use std::any::Any;

/// Supports conversion to `Any`. Traits to be extended by `impl_downcast!` must extend `Downcast`.
pub trait Downcast: Any {
    /// Convert `Box<Trait>` (where `Trait: Downcast`) to `Box<Any>`. `Box<Any>` can then be
    /// further `downcast` into `Box<ConcreteType>` where `ConcreteType` implements `Trait`.
    fn into_any(self: Box<Self>) -> Box<Any>;
    /// Convert `&Trait` (where `Trait: Downcast`) to `&Any`. This is needed since Rust cannot
    /// generate `&Any`'s vtable from `&Trait`'s.
    fn as_any(&self) -> &Any;
    /// Convert `&mut Trait` (where `Trait: Downcast`) to `&Any`. This is needed since Rust cannot
    /// generate `&mut Any`'s vtable from `&mut Trait`'s.
    fn as_any_mut(&mut self) -> &mut Any;
}

impl<T: Any> Downcast for T {
    fn into_any(self: Box<Self>) -> Box<Any> { self }
    fn as_any(&self) -> &Any { self }
    fn as_any_mut(&mut self) -> &mut Any { self }
}

/// Adds downcasting support to traits that extend `downcast::Downcast` by defining forwarding
/// methods to the corresponding implementations on `std::any::Any` in the standard library.
///
/// See https://users.rust-lang.org/t/how-to-create-a-macro-to-impl-a-provided-type-parametrized-trait/5289
/// for why this is implemented this way to support templatized traits.
#[macro_export]
macro_rules! impl_downcast {
    (@impl_full
        $trait_:ident [$($param_types:tt)*]
        for [$($forall_types:ident),*]
        where [$($preds:tt)*]
    ) => {
        impl_downcast! {
            @inject_where
                [impl<$($forall_types),*> $trait_<$($param_types)*>]
                types [$($forall_types),*]
                where [$($preds)*]
                [{
                    impl_downcast! { @impl_body $trait_ [$($param_types)*] }
                }]
        }
    };

    (@impl_body $trait_:ident [$($types:tt)*]) => {
        /// Returns true if the trait object wraps an object of type `__T`.
        #[inline]
        pub fn is<__T: $trait_<$($types)*>>(&self) -> bool {
            $crate::Downcast::as_any(self).is::<__T>()
        }
        /// Returns a boxed object from a boxed trait object if the underlying object is of type
        /// `__T`. Returns the original boxed trait if it isn't.
        #[inline]
        pub fn downcast<__T: $trait_<$($types)*>>(
            self: ::std::boxed::Box<Self>
        ) -> ::std::result::Result<::std::boxed::Box<__T>, ::std::boxed::Box<Self>> {
            if self.is::<__T>() {
                Ok($crate::Downcast::into_any(self).downcast::<__T>().unwrap())
            } else {
                Err(self)
            }
        }
        /// Returns a reference to the object within the trait object if it is of type `__T`, or
        /// `None` if it isn't.
        #[inline]
        pub fn downcast_ref<__T: $trait_<$($types)*>>(&self) -> ::std::option::Option<&__T> {
            $crate::Downcast::as_any(self).downcast_ref::<__T>()
        }
        /// Returns a mutable reference to the object within the trait object if it is of type
        /// `__T`, or `None` if it isn't.
        #[inline]
        pub fn downcast_mut<__T: $trait_<$($types)*>>(&mut self) -> ::std::option::Option<&mut __T> {
            $crate::Downcast::as_any_mut(self).downcast_mut::<__T>()
        }
    };

    (@inject_where [$($before:tt)*] types [] where [] [$($after:tt)*]) => {
        impl_downcast! { @as_item $($before)* $($after)* }
    };

    (@inject_where [$($before:tt)*] types [$($types:ident),*] where [] [$($after:tt)*]) => {
        impl_downcast! {
            @as_item
                $($before)*
                where $( $types: ::std::any::Any + 'static ),*
                $($after)*
        }
    };
    (@inject_where [$($before:tt)*] types [$($types:ident),*] where [$($preds:tt)+] [$($after:tt)*]) => {
        impl_downcast! {
            @as_item
                $($before)*
                where
                    $( $types: ::std::any::Any + 'static, )*
                    $($preds)*
                $($after)*
        }
    };

    (@as_item $i:item) => { $i };

    // No type parameters.
    ($trait_:ident   ) => { impl_downcast! { @impl_full $trait_ [] for [] where [] } };
    ($trait_:ident <>) => { impl_downcast! { @impl_full $trait_ [] for [] where [] } };
    // Type parameters.
    ($trait_:ident < $($types:ident),* >) => {
        impl_downcast! { @impl_full $trait_ [$($types),*] for [$($types),*] where [] }
    };
    // Type parameters and where clauses.
    ($trait_:ident < $($types:ident),* > where $($preds:tt)+) => {
        impl_downcast! { @impl_full $trait_ [$($types),*] for [$($types),*] where [$($preds)*] }
    };
    // Associated types.
    ($trait_:ident assoc $($atypes:ident),*) => {
        impl_downcast! { @impl_full $trait_ [$($atypes = $atypes),*] for [$($atypes),*] where [] }
    };
    // Associated types and where clauses.
    ($trait_:ident assoc $($atypes:ident),* where $($preds:tt)+) => {
        impl_downcast! { @impl_full $trait_ [$($atypes = $atypes),*] for [$($atypes),*] where [$($preds)*] }
    };
    // Type parameters and associated types.
    ($trait_:ident < $($types:ident),* > assoc $($atypes:ident),*) => {
        impl_downcast! {
            @impl_full
                $trait_ [$($types),*, $($atypes = $atypes),*]
                for [$($types),*, $($atypes),*]
                where []
        }
    };
    // Type parameters, associated types, and where clauses.
    ($trait_:ident < $($types:ident),* > assoc $($atypes:ident),* where $($preds:tt)+) => {
        impl_downcast! {
            @impl_full
                $trait_ [$($types),*, $($atypes = $atypes),*]
                for [$($types),*, $($atypes),*]
                where [$($preds)*]
        }
    };
    // Concretely-parametrized types.
    (concrete $trait_:ident < $($types:ident),* >) => {
        impl_downcast! { @impl_full $trait_ [$($types),*] for [] where [] }
    };
    // Concretely-associated types types.
    (concrete $trait_:ident assoc $($atypes:ident = $aty:ty),*) => {
        impl_downcast! { @impl_full $trait_ [$($atypes = $aty),*] for [] where [] }
    };
    // Concretely-parametrized types with concrete associated types.
    (concrete $trait_:ident < $($types:ident),* > assoc $($atypes:ident = $aty:ty),*) => {
        impl_downcast! { @impl_full $trait_ [$($types),*, $($atypes = $aty),*] for [] where [] }
    };
}


#[cfg(test)]
mod test {
    macro_rules! test_mod {
        (
            $test_name:ident,
            trait $base_trait:path { $($base_impl:tt)* },
            type $base_type:ty,
            { $($def:tt)+ }
        ) => {
            mod $test_name {
                use super::super::Downcast;

                // A trait that can be downcast.
                $($def)*

                // Concrete type implementing Base.
                #[derive(Debug)]
                struct Foo(u32);
                impl $base_trait for Foo { $($base_impl)* }
                #[derive(Debug)]
                struct Bar(f64);
                impl $base_trait for Bar { $($base_impl)* }

                // Functions that can work on references to Base trait objects.
                fn get_val(base: &::std::boxed::Box<$base_type>) -> u32 {
                    match base.downcast_ref::<Foo>() {
                        Some(val) => val.0,
                        None => 0
                    }
                }
                fn set_val(base: &mut ::std::boxed::Box<$base_type>, val: u32) {
                    if let Some(foo) = base.downcast_mut::<Foo>() {
                        foo.0 = val;
                    }
                }

                #[test]
                fn test() {
                    let mut base: ::std::boxed::Box<$base_type> = ::std::boxed::Box::new(Foo(42));
                    assert_eq!(get_val(&base), 42);

                    // Try sequential downcasts.
                    if let Some(foo) = base.downcast_ref::<Foo>() {
                        assert_eq!(foo.0, 42);
                    } else if let Some(bar) = base.downcast_ref::<Bar>() {
                        assert_eq!(bar.0, 42.0);
                    }

                    set_val(&mut base, 6*9);
                    assert_eq!(get_val(&base), 6*9);

                    assert!(base.is::<Foo>());

                    // Fail to convert Box<Base> into Box<Bar>.
                    let res = base.downcast::<Bar>();
                    assert!(res.is_err());
                    let base = res.unwrap_err();
                    // Convert Box<Base> into Box<Foo>.
                    assert_eq!(
                        6*9, base.downcast::<Foo>().map_err(|_| "Shouldn't happen.").unwrap().0);
                }
            }
        };

        (
            $test_name:ident,
            trait $base_trait:path { $($base_impl:tt)* },
            { $($def:tt)+ }
        ) => {
            test_mod! {
                $test_name, trait $base_trait { $($base_impl:tt)* }, type $base_trait, { $($def)* }
            }
        }
    }

    test_mod!(non_generic, trait Base {}, {
        trait Base: Downcast {}
        impl_downcast!(Base);
    });

    test_mod!(generic, trait Base<u32> {}, {
        trait Base<T>: Downcast {}
        impl_downcast!(Base<T>);
    });

    test_mod!(constrained_generic, trait Base<u32> {}, {
        // Should work even if standard objects in the prelude are aliased to something else.
        #[allow(dead_code)] struct Box;
        #[allow(dead_code)] struct Option;
        #[allow(dead_code)] struct Result;
        trait Base<T: Copy>: Downcast {}
        impl_downcast!(Base<T> where T: Copy);
    });

    test_mod!(associated, trait Base { type H = f32; }, type Base<H=f32>, {
        trait Base: Downcast { type H; }
        impl_downcast!(Base assoc H);
    });

    test_mod!(constrained_associated, trait Base { type H = f32; }, type Base<H=f32>, {
        trait Base: Downcast { type H: Copy; }
        impl_downcast!(Base assoc H where H: Copy);
    });

    test_mod!(param_and_associated, trait Base<u32> { type H = f32; }, type Base<u32, H=f32>, {
        trait Base<T>: Downcast { type H; }
        impl_downcast!(Base<T> assoc H);
    });

    test_mod!(constrained_param_and_associated, trait Base<u32> { type H = f32; }, type Base<u32, H=f32>, {
        trait Base<T: Clone>: Downcast { type H: Copy; }
        impl_downcast!(Base<T> assoc H where T: Clone, H: Copy);
    });

    test_mod!(concrete_parametrized, trait Base<u32> {}, {
        trait Base<T>: Downcast {}
        impl_downcast!(concrete Base<u32>);
    });

    test_mod!(concrete_associated, trait Base { type H = u32; }, type Base<H=u32>, {
        trait Base: Downcast { type H; }
        impl_downcast!(concrete Base assoc H=u32);
    });

    test_mod!(concrete_parametrized_associated, trait Base<u32> { type H = f32; }, type Base<u32, H=f32>, {
        trait Base<T>: Downcast { type H; }
        impl_downcast!(concrete Base<u32> assoc H=f32);
    });
}
