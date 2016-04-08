///! Rust enums are great for types where all variations are known beforehand. But in
///! the case where you want to implement a container of user-defined types, an
///! open-ended type like a trait object is needed. In some cases, it is useful to
///! cast the trait object back into its original concrete type to access additional
///! functionality and performant inlined implementations.
///! 
///! `downcast` is an exercise in adding basic down-casting support to trait objects
///! just as [MOPA](https://crates.io/crates/mopa/) and
///! [downcast](https://crates.io/crates/downcast/) do while avoiding unsafe code and
///! without replicating the behavior in the standard library. This is at the
///! (negligible) expense of adding two methods to the down-castable trait's vtable.
///! 
///! To make a trait downcastable, make it extend the `downcast::Downcast` trait and
///! invoke `impl_downcast!` on it as follows:
///!
///! ```rust
///! # #[macro_use]
///! # extern crate downcast_rs;
///! # use downcast_rs::Downcast;
///! trait Trait: Downcast {}
///! impl_downcast!(Trait);
///!
///! // or
///! 
///! trait TraitGeneric<T>: Downcast {}
///! impl_downcast!(TraitGeneric<T>);
///!
///! // or
///!
///! trait TraitGenericConstrained<T: Copy>: Downcast {}
///! impl_downcast!(TraitGenericConstrained<T> where T: Copy);
///!
///! // or
///!
///! // Use this variant when specifying concrete type parameters.
///! trait TraitGenericConcrete<T: Copy>: Downcast {}
///! impl_downcast!(concrete TraitGenericConcrete<u32>);
///! #
///! # fn main() {}
///! ```
///! 
///! # Example without generics
///!
///! ```rust
///! #[macro_use]
///! extern crate downcast_rs;
///! use downcast_rs::Downcast;
///! 
///! // To create a trait with downcasting methods, extend `Downcast` and run
///! // impl_downcast!() on the trait.
///! trait Base: Downcast {}
///! impl_downcast!(Base);
///! 
///! // Concrete type implementing Base.
///! struct Foo(u32);
///! impl Base for Foo {}
///! 
///! fn main() {
///!     // Create a trait object.
///!     let mut base: Box<Base> = Box::new(Foo(42));
///! 
///!     // Downcast to Foo.
///!     assert_eq!(base.downcast_ref::<Foo>().unwrap().0, 42);
///! }
///! ```
///!
///! # Example with a generic trait
///!
///! ```rust
///! #[macro_use]
///! extern crate downcast_rs;
///! use downcast_rs::Downcast;
///! 
///! // To create a trait with downcasting methods, extend `Downcast` and run
///! // impl_downcast!() on the trait.
///! trait Base<T>: Downcast {}
///! impl_downcast!(Base<T>);
///! 
///! // Concrete type implementing Base.
///! struct Foo(u32);
///! impl Base<u32> for Foo {}
///! 
///! fn main() {
///!     // Create a trait object.
///!     let mut base: Box<Base<u32>> = Box::new(Foo(42));
///! 
///!     // Downcast to Foo.
///!     assert_eq!(base.downcast_ref::<Foo>().unwrap().0, 42);
///! }
///! ```

use std::any::Any;

/// Supports conversion to `Any`. Traits to be extended by `impl_downcast!` must extend `Downcast`.
pub trait Downcast: Any {
    fn as_any(&self) -> &Any;
    fn as_any_mut(&mut self) -> &mut Any;
}

impl<T: Any> Downcast for T {
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
    (@$trait_:ident) => {
        impl_downcast! {
            @as_item
            impl $trait_ {
                impl_downcast! { @impl_body @$trait_ [] }
            }
        }
    };

    (@$trait_:ident [$($args:ident,)*]) => {
        /// Implementation for a trait with generic parameters passed.
        /// In its current state, this will not work if the trait requires any constraints on the
        /// type parameters other than `::std::any::Any` and `'static`.
        impl_downcast! {
            @as_item
            impl<$($args),*> $trait_<$($args),*>
                where $( $args: ::std::any::Any + 'static ),*
            {
                impl_downcast! { @impl_body @$trait_ [$($args,)*] }
            }
        }
    };

    (@$trait_:ident [$($args:ident,)*] where [$($preds:tt)+]) => {
        /// Implementation for a trait with generic parameters passed.
        /// In its current state, this will not work if the trait requires any constraints on the
        /// type parameters other than `::std::any::Any` and `'static`.
        impl_downcast! {
            @as_item
            impl<$($args),*> $trait_<$($args),*>
                where $( $args: ::std::any::Any + 'static, )*
                      $($preds)*
            {
                impl_downcast! { @impl_body @$trait_ [$($args,)*] }
            }
        }
    };

    (concrete @$trait_:ident [$($args:ident,)*]) => {
        /// Implementation for a trait with concrete types passed.
        impl_downcast! {
            @as_item
            impl $trait_<$($args),*> {
                impl_downcast! { @impl_body @$trait_ [$($args,)*] }
            }
        }
    };

    (@impl_body @$trait_:ident [$($args:ident,)*]) => {
        /// Returns true if the boxed type is the same as `__T`.
        #[inline]
        pub fn is<__T: $trait_<$($args),*>>(&self) -> bool {
            $crate::Downcast::as_any(self).is::<__T>()
        }
        /// Returns a reference to the boxed value if it is of type `__T`, or
        /// `None` if it isn't.
        #[inline]
        pub fn downcast_ref<__T: $trait_<$($args),*>>(&self) -> Option<&__T> {
            $crate::Downcast::as_any(self).downcast_ref::<__T>()
        }
        /// Returns a mutable reference to the boxed value if it is of type
        /// `__T`, or `None` if it isn't.
        #[inline]
        pub fn downcast_mut<__T: $trait_<$($args),*>>(&mut self) -> Option<&mut __T> {
            $crate::Downcast::as_any_mut(self).downcast_mut::<__T>()
        }
    };

    (@as_item $i:item) => { $i };

    ($trait_:ident <>) => { impl_downcast! { @$trait_ } };
    ($trait_:ident < $($args:ident),* $(,)* >) => { impl_downcast! { @$trait_ [$($args,)*] } };
    ($trait_:ident) => { impl_downcast! { @$trait_ } };
    (concrete $trait_:ident < $($args:ident),* $(,)* >) => {
        impl_downcast! { concrete @$trait_ [$($args,)*] }
    };
    ($trait_:ident < $($args:ident),* $(,)* > where $($preds:tt)+) => {
        impl_downcast! { @$trait_ [$($args,)*] where [$($preds)*] }
    };
}


#[cfg(test)]
mod test {

    mod non_generic {
        use super::super::Downcast;

        // A trait that can be downcast.
        trait Base: Downcast {}
        impl_downcast!(Base);

        // Concrete type implementing Base.
        struct Foo(u32);
        impl Base for Foo {}

        // Functions that can work on references to Base trait objects.
        fn get_val(base: &Box<Base>) -> u32 {
            match base.downcast_ref::<Foo>() {
                Some(val) => val.0,
                None => 0
            }
        }
        fn set_val(base: &mut Box<Base>, val: u32) {
            if let Some(foo) = base.downcast_mut::<Foo>() {
                foo.0 = val;
            }
        }

        #[test]
        fn test() {
            let mut base: Box<Base> = Box::new(Foo(42));
            assert_eq!(get_val(&base), 42);

            set_val(&mut base, 6*9);
            assert_eq!(get_val(&base), 6*9);

            assert!(base.is::<Foo>());
        }
    }

    mod generic {
        use super::super::Downcast;

        // A trait that can be downcast.
        trait Base<T>: Downcast {}
        impl_downcast!(Base<T>);

        // Concrete type implementing Base.
        struct Foo(u32);
        impl Base<u32> for Foo {}

        // Functions that can work on references to Base trait objects.
        fn get_val(base: &Box<Base<u32>>) -> u32 {
            match base.downcast_ref::<Foo>() {
                Some(val) => val.0,
                None => 0
            }
        }
        fn set_val(base: &mut Box<Base<u32>>, val: u32) {
            if let Some(foo) = base.downcast_mut::<Foo>() {
                foo.0 = val;
            }
        }

        #[test]
        fn test() {
            let mut base: Box<Base<u32>> = Box::new(Foo(42));
            assert_eq!(get_val(&base), 42);

            set_val(&mut base, 6*9);
            assert_eq!(get_val(&base), 6*9);

            assert!(base.is::<Foo>());
        }
    }

    mod constrained_generic {
        use super::super::Downcast;

        // A trait that can be downcast.
        trait Base<T: Copy>: Downcast {}
        impl_downcast!(Base<T> where T: Copy);

        // Concrete type implementing Base.
        struct Foo(u32);
        impl Base<u32> for Foo {}

        // Functions that can work on references to Base trait objects.
        fn get_val(base: &Box<Base<u32>>) -> u32 {
            match base.downcast_ref::<Foo>() {
                Some(val) => val.0,
                None => 0
            }
        }
        fn set_val(base: &mut Box<Base<u32>>, val: u32) {
            if let Some(foo) = base.downcast_mut::<Foo>() {
                foo.0 = val;
            }
        }

        #[test]
        fn test() {
            let mut base: Box<Base<u32>> = Box::new(Foo(42));
            assert_eq!(get_val(&base), 42);

            set_val(&mut base, 6*9);
            assert_eq!(get_val(&base), 6*9);

            assert!(base.is::<Foo>());
        }
    }

    mod concrete {
        use super::super::Downcast;

        // A trait that can be downcast.
        trait Base<T>: Downcast {}
        impl_downcast!(concrete Base<u32>);

        // Concrete type implementing Base.
        struct Foo(u32);
        impl Base<u32> for Foo {}

        // Functions that can work on references to Base trait objects.
        fn get_val(base: &Box<Base<u32>>) -> u32 {
            match base.downcast_ref::<Foo>() {
                Some(val) => val.0,
                None => 0
            }
        }
        fn set_val(base: &mut Box<Base<u32>>, val: u32) {
            if let Some(foo) = base.downcast_mut::<Foo>() {
                foo.0 = val;
            }
        }

        #[test]
        fn test() {
            let mut base: Box<Base<u32>> = Box::new(Foo(42));
            assert_eq!(get_val(&base), 42);

            set_val(&mut base, 6*9);
            assert_eq!(get_val(&base), 6*9);

            assert!(base.is::<Foo>());
        }
    }

}
