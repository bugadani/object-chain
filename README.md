Object chain - build ad-hoc structures
======================================

Object chains are static objects whose type depends on the objects you store in them.
This data structure is useful if you need to collect different objects that implement a common
functionality and you don't want heap allocation.

To get started, you need to create a `Chain` object by passing it your first object.
Use the `append` method to add more objects to your chain.
If you need to pass the chain around, you can use `impl ChainElement` or, if you need to be
explicit about the type, the `chain!` macro.

If you want to access the elements inside, you'll need to implement a common trait for your objects
and an accessor interface for `Chain` and `Link`. You can see an example in the source code.
