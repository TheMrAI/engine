mod celled;
mod non_celled;

// Just a quick experiment trying out how one may access multiple elements of some
// internal container, without locking down the whole outer container.
//
// Borrow checking in Rust is rather strict. One cannot describe just access patterns where
// non-overlapping sub sections are borrowed from a given type. Either the whole structure is
// locked for multiple non-mutable access of one mutable one. There is simply no other option.
//
// One can start hacking around these restrictions with various utilities from std::cell, but those
// alone are barely usable. Since they also rely on references from their containing scope, they would
// also lock down everything as any other simple referencing would.
// To step around the issue, they must be wrapped in a type that can be passed around as a value. That
// can be copied as a value, which has no lifetime dependencies. This is most commonly an 'Rc' or 'Arc'.
// A ref counted shared pointer. With this, the data pointed to by the 'Rc' can be accessed from any
// 'Rc' instance. The 'Rc' has no lifetime dependencies. The stored `RefCell` can be accessed and produce
// reference using the lifetime of the `Rc` itself.
// A clever solution to the borrow checker. This also allows the granular and safe access of all components within
// internal containers, without having to lock down the external one.
//
// The problem I have with this design is that it introduces a lot of indirections all of which carry their
// baggage. Both in terms of memory and runtime shortcomings.
// Additionally they encode 'threadedness' into the types they are used in. `Rc` only works in single threaded
// scenarios while `Arc` can work in multithreaded ones too, but it is more costly to use in single threaded ones.
//
// Seems to me that this pattern won't be good enough to be used in an ECS implementation. The safety that is provided
// for the user feels less important than the performance loss we will incur by the additional management.
