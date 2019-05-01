# Mapred in Rust
## Goal of the exercise

Write a program counting the frequency of each word in a large
document. Split the task on different threads, effectively
implementing map reduce (hence the name). The words should be output
in lexical order.

## Performance

I have an implementation in C, with a custom (unbalanced) binary
search tree as the primary data structure. I can use it for comparison
with this Rust implementation, that uses the standard library BTreeMap
search tree. Both implementations mmap the entire file, and merge sort
the results from individual threads. (An idea could be to mmap as many
regions as there are threads, madvise'ing the kernel that they will be
read sequentially.)

I expect the BTreeMap implementation to be superior to my custom
binary search tree implementation in C. However the C implementation
so far is faster on both single and multiple threads, for reasons that
are not yet clear (I am testing with random data so that it doesn't
matter that the tree in C is unbalanced). Failing to guess, my next
best lead would be to profile in detail both implementations.

(I tried to cut the word from chunk with `unsafe {
chunk.get_unchecked(start..oneafter) }` to avoid bound checking, but
with limited improvement on execution speed)

## Passing references to threads in Rust

References cannot be passed to new threads. If the reference points to
something on the parent's stack, then the reference would become
invalid once the function returns.

The standard thread library solves this by enforcing any reference
passed to a new thread to have static lifetime :

```
pub fn spawn<F, T>(f: F) -> JoinHandle<T> where
    F: FnOnce() -> T, F: Send + 'static, T: Send + 'static
```

So we won't ever be able to have threads using a reference from the
parent, other than that reference having a static lifetime (this rules
out a buffer read from a file). That's what's in the standard library.
However, we have libraries that implement threads differently. Here is
the equivalent signature of spawn, in the scoped threadpool library :

```
fn execute_<F>(&self, f: F) where F: FnOnce() + Send + 'scope {
```

We're no longer tied to the special static lifetime. However, scoped
threadpool uses the standard library threads internally. (It uses
unsafe and type magic that flies over my head for now.) There's
nothing wrong with that as long as the implementation guarantees
overall safety.

Here I have not taken up the reference approach, but instead sticked
to a _concrete_ type (as opposed to a reference). First it was
`Vec<u8>` as a buffer read from file, then `Mmap` from the memmap
crate, and finally `AsRef<[u8]>`, which is a trait whose behavior
captures both the `Vec<u8>` and `Mmap` implementations. (Apparently
concrete types satisfy 'static if they don't themselves contain
references.)

Explanation for static lifetime : https://users.rust-lang.org/t/why-does-thread-spawn-need-static-lifetime-for-generic-bounds/4541

Sending slices to threads : https://stackoverflow.com/questions/26477757/how-do-you-send-slices-of-a-vec-to-a-task-in-rust
