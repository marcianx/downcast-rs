# downcast-rs

Rust enums are great for types where all variations are known beforehand. But in
the case where you want to implement a container of user-defined types, an
open-ended type like a trait object is needed. In some cases, it is useful to
cast the trait object back into its original concrete type to access additional
functionality and performant inlined implementations.

`downcast-rs` adds basic downcasting support to trait objects, supporting **type
parameters**, **associated types**, and **constraints**.

To make a trait downcastable, make it extend the `downcast::Downcast` trait and
invoke `impl_downcast!` on it as follows:

```rust
trait Trait: Downcast {}
impl_downcast!(Trait);

// With type parameters.
trait TraitGeneric1<T>: Downcast {}
impl_downcast!(TraitGeneric1<T>);

// With associated types.
trait TraitGeneric2: Downcast { type G; type H; }
impl_downcast!(TraitGeneric2 assoc G, H);

// With constraints on types.
trait TraitGeneric3<T: Copy>: Downcast {
    type H: Clone;
}
impl_downcast!(TraitGeneric3<T> assoc H where T: Copy, H: Clone);

// With concrete types.
trait TraitConcrete1<T: Copy>: Downcast {}
impl_downcast!(concrete TraitConcrete1<u32>);

trait TraitConcrete2<T: Copy>: Downcast { type H; }
impl_downcast!(concrete TraitConcrete2<u32> assoc H=f64);
```

## Example without generics

```rust
#[macro_use]
extern crate downcast_rs;
use downcast_rs::Downcast;

// To create a trait with downcasting methods, extend `Downcast` and run
// impl_downcast!() on the trait.
trait Base: Downcast {}
impl_downcast!(Base);

// Concrete types implementing Base.
#[derive(Debug)]
struct Foo(u32);
impl Base for Foo {}
#[derive(Debug)]
struct Bar(f64);
impl Base for Bar {}

fn main() {
    // Create a trait object.
    let mut base: Box<Base> = Box::new(Foo(42));

    // Try sequential downcasts.
    if let Some(foo) = base.downcast_ref::<Foo>() {
        assert_eq!(foo.0, 42);
    } else if let Some(bar) = base.downcast_ref::<Bar>() {
        assert_eq!(bar.0, 42.0);
    }

    assert!(base.is::<Foo>());

    // Fail to convert Box<Base> into Box<Bar>.
    let res = base.downcast::<Bar>();
    assert!(res.is_err());
    let base = res.unwrap_err();
    // Convert Box<Base> into Box<Foo>.
    assert_eq!(42, base.downcast::<Foo>().map_err(|_| "Shouldn't happen.").unwrap().0);
}
```

## Example with a generic trait with associated types and constraints

```rust
#[macro_use]
extern crate downcast_rs;
use downcast_rs::Downcast;

// To create a trait with downcasting methods, extend `Downcast` and run
// impl_downcast!() on the trait.
trait Base<T: Clone>: Downcast { type H: Copy; }
impl_downcast!(Base<T> assoc H where T: Clone, H: Copy);
// or: impl_downcast!(concrete Base<u32> assoc H=f32)

// Concrete types implementing Base.
struct Foo(u32);
impl Base<u32> for Foo { type H = f32; }
struct Bar(f64);
impl Base<u32> for Bar { type H = f32; }

fn main() {
    // Create a trait object.
    let mut base: Box<Base<u32, H=f32>> = Box::new(Bar(42.0));

    // Try sequential downcasts.
    if let Some(foo) = base.downcast_ref::<Foo>() {
        assert_eq!(foo.0, 42);
    } else if let Some(bar) = base.downcast_ref::<Bar>() {
        assert_eq!(bar.0, 42.0);
    }

    assert!(base.is::<Bar>());
}
```

## License

Copyright 2015, Ashish Myles.
This software is dual-licensed under the [MIT](LICENSE-MIT) and
[Apache 2.0](LICENSE-APACHE) licenses.
