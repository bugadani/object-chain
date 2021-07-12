//! Create static chains of objects with different types.
//!
//! In general, the chain starts (or ends, depending on your view) with a `Chain` element
//! and is built up from any number of `Link`s.
//!
//! This basic structure only allows you
//! to query the number of elements, but you can implement a more useful trait for both `Link` and
//! `Chain` to make this structure more useful. For an example, check the
//! `test_accessing_elements_with_common_interface` test in the source code.
#![no_std]
mod private {
    pub trait Sealed {}

    impl<V> Sealed for super::Chain<V> {}
    impl<V, C: super::ChainElement> Sealed for super::Link<V, C> {}
}

/// A generic chain element
pub trait ChainElement: private::Sealed {
    type Inner;

    /// Append an object to the chain
    #[inline]
    fn append<T>(self, item: T) -> Link<T, Self>
    where
        Self: Sized,
    {
        Link {
            object: item,
            parent: self,
        }
    }

    /// Return the number of objects linked to this chain element
    fn len(&self) -> usize;

    fn get(&self) -> &Self::Inner;

    fn get_mut(&mut self) -> &mut Self::Inner;
}

/// This piece of the chain contains some object
#[derive(Clone, Copy)]
pub struct Link<V, C>
where
    C: ChainElement,
{
    /// The rest of the object chain
    pub parent: C,

    /// The current object
    pub object: V,
}

impl<V, VC> ChainElement for Link<V, VC>
where
    VC: ChainElement,
{
    type Inner = V;

    #[inline]
    fn len(&self) -> usize {
        self.parent.len() + 1
    }

    fn get(&self) -> &Self::Inner {
        &self.object
    }

    fn get_mut(&mut self) -> &mut Self::Inner {
        &mut self.object
    }
}

/// This piece marks the end of a chain.
#[derive(Clone, Copy)]
pub struct Chain<V> {
    /// The wrapped object.
    pub object: V,
}

impl<V> Chain<V> {
    /// Creates a new [`Chain`] by wrapping the given object.
    pub const fn new(object: V) -> Self {
        Self { object }
    }
}

impl<V> ChainElement for Chain<V> {
    type Inner = V;

    #[inline]
    fn len(&self) -> usize {
        1
    }

    fn get(&self) -> &Self::Inner {
        &self.object
    }

    fn get_mut(&mut self) -> &mut Self::Inner {
        &mut self.object
    }
}

/// Internal implementation of chain macro
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! chain_impl {
    ($x:ty) => {
        Chain<$x>
    };
    ($x:ty,) => {
        Chain<$x>
    };
    ($x:ty, $($rest:tt)+) => {
        Link<$x, chain_impl! { $($rest)+ }>
    };
}

/// Reverse the argument list to generate object chain
#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! reverse {
    ([] $($reversed:tt)+) => {
        chain_impl! { $($reversed)+ }
    };
    ([$first:ty] $($reversed:tt)*) => {
        reverse! { [ ] $first, $($reversed)* }
    };
    ([$first:ty, $($rest:ty),*] $($reversed:tt)*) => {
        reverse! { [ $($rest),* ] $first, $($reversed)* }
    };
}

/// Creates an object chain from the argument types.
///
/// Using this macro is completely optional but it reduces the boilerplate required to describe
/// the type of an object chain.
///
/// # Example:
///
/// Instead of writing this...
///
/// ```rust
/// # struct A;
/// # struct B;
/// # struct C;
/// use object_chain::{Chain, Link};
/// type ABC = Link<C, Link<B, Chain<A>>>;
/// ```
///
/// ... the `chain!` macro allows you to write this:
///
/// ```rust
/// # struct A;
/// # struct B;
/// # struct C;
/// use object_chain::{Chain, Link, chain};
/// type ABC = chain![A, B, C];
/// ```
///
/// Note also how the order of types follows the type of objects in the chain instead of being
/// reversed.
#[macro_export(local_inner_macros)]
macro_rules! chain {
    [$($types:ty),+] => {
        reverse!{ [ $($types),+ ] }
    };
}

#[cfg(test)]
#[allow(dead_code)]
mod test {
    use super::*;
    use core::marker::PhantomData;

    struct CompileTest {
        chain1: chain![u8],
        generic_in_chain: chain![Generic<'static, u32>],
        chain: chain![u8, u16, u32],
    }

    struct Generic<'a, T> {
        field: PhantomData<&'a T>,
    }

    #[test]
    pub fn test() {
        fn f(_obj_chain: &chain![u8, u16, u32]) {}

        let test = CompileTest {
            chain1: Chain::new(0),
            generic_in_chain: Chain::new(Generic { field: PhantomData }),
            chain: Chain::new(0u8).append(1u16).append(2u32),
        };

        f(&test.chain);
    }

    #[test]
    pub fn test_count() {
        assert_eq!(1, Chain::new(0).len());
        assert_eq!(3, Chain::new(0u8).append(1u16).append(2u32).len());
    }

    #[test]
    pub fn test_accessing_elements_with_common_interface() {
        // 1: First, we need to implement a common interface for all of our objects' types
        trait AsU8 {
            fn as_u8(&self) -> u8;
        }

        impl AsU8 for u8 {
            fn as_u8(&self) -> u8 {
                *self
            }
        }

        impl AsU8 for u16 {
            fn as_u8(&self) -> u8 {
                *self as u8
            }
        }

        // 2: Next, we need to implement an accessor interface
        trait ReadableAsU8 {
            fn read_from(&self, index: usize) -> &dyn AsU8;
        }

        impl<T: AsU8> ReadableAsU8 for Chain<T> {
            fn read_from(&self, index: usize) -> &dyn AsU8 {
                assert!(index == 0, "Out of bounds access!");
                &self.object
            }
        }

        impl<T: AsU8, CE: ChainElement<Inner = impl AsU8> + ReadableAsU8> ReadableAsU8 for Link<T, CE> {
            fn read_from(&self, index: usize) -> &dyn AsU8 {
                if index == self.len() - 1 {
                    &self.object
                } else {
                    self.parent.read_from(index - 1)
                }
            }
        }

        // Through the accessor interface we can access any of our objects, but only through the
        // common interface we defined in step 1.
        fn do_test(obj_chain: &impl ReadableAsU8) {
            assert_eq!(2, obj_chain.read_from(1).as_u8());
        }

        do_test(&Chain::new(1u8).append(2u16));
    }
}
