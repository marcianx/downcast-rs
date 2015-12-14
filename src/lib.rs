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
///! invoke `downcast_impl!` on it as follows:
///!
///! ```rust
///! trait Trait: Downcast {}
///! downcast_impl!(Trait);
///! ```
///! 
///! # Example
///!
///! ```rust
///! #[macro_use]
///! extern crate downcast_rs;
///! use downcast_rs::Downcast;
///! 
///! // To create a trait with downcasting methods, extend `Downcast` and run
///! // downcast_impl!() on the trait.
///! trait Base: Downcast {}
///! downcast_impl!(Base);
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

use std::any::Any;

/// Supports conversion to `Any`. Traits to be extended by `downcast_impl!` must extend `Downcast`.
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
#[macro_export]
macro_rules! downcast_impl {
    ($trait_:ident) => {
        impl $trait_ {
            /// Returns true if the boxed type is the same as `T`.
            #[inline]
            pub fn is<T: $trait_>(&self) -> bool {
                $crate::Downcast::as_any(self).is::<T>()
            }
            /// Returns a reference to the boxed value if it is of type `T`, or
            /// `None` if it isn't.
            #[inline]
            pub fn downcast_ref<T: $trait_>(&self) -> Option<&T> {
                $crate::Downcast::as_any(self).downcast_ref::<T>()
            }
            /// Returns a mutable reference to the boxed value if it is of type
            /// `T`, or `None` if it isn't.
            #[inline]
            pub fn downcast_mut<T: $trait_>(&mut self) -> Option<&mut T> {
                $crate::Downcast::as_any_mut(self).downcast_mut::<T>()
            }
        }
    }
}


#[cfg(test)]
mod test {
    use super::Downcast;

    // A trait that can be downcast.
    trait Base: Downcast {}
    downcast_impl!(Base);

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
