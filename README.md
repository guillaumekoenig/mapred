# Passing reference to thread

I cannot pass a reference (borrowed value) to a thread. The compiler
has no way of knowing if the reference will remain valid for both the
lifetime of the parent (which owns the reference), and that of the
child. Ie the parent can die first, or the child can die first.

The standard thread library solves this by enforcing any reference to
be static :

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
threadpool uses the standard library threads internally. Not sure how
it does it at this point.
