# Hierrorchy

Hierrorchy is a proc-macro library to simplify the creation of error hierarchies (hence the name), in a tree-like structure.

This crate is based on two concepts:
- Leaves: base errors which can occur during the execution of a program
([`hierrorchy::error_leaf`](macro@error_leaf)).
- Nodes: errors which source can be a leaf or another node
([`hierrorchy::error_node`](macro@error_node)).

As nodes are "just" containers for other errors, they are `enum`s with a variant for each type
of error they can contain; while leaves, which must be as open as possible, are `struct`s.

## Examples
### Example of an error leaf
Error leaves are declared by adding an attribute to a struct definition (see
[`hierrorchy::error_leaf`](macro@error_leaf) documentation for details on its configuration):
```
use hierrorchy::error_leaf;

#[error_leaf("My error")]
struct MyError {}
```

The attribute adds the implementation of [`std::fmt::Display`] and [`std::error::Error`], thus
writing the snippet of code above is equivalent to wrinting the following code:
```
#[derive(Debug)]
struct MyError{}

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", "My error")
    }
}

impl std::error::Error for MyError {}
```

As you can see from the snippet above, [`hierrorchy::error_leaf`](macro@error_leaf) adds the attribute for
deriving the [`std::fmt::Debug`] implementation, as it is required by [`std::error::Error`].

### Example of an error node
Error nodes are declared by the function-like macro [`hierrorchy::error_node`](macro@error_node):
```
use hierrorchy::{error_leaf,error_node};
use std::error::Error;

#[error_leaf("My error")]
struct MyError {}

error_node! { type MyErrorNode<MyError> = "my error node" }
```

This snippet is equivalent to:
```
use hierrorchy::error_leaf;
use std::error::Error;

#[error_leaf("My error")]
struct MyError {}

#[derive(Debug)]
enum MyErrorNode {
    Variant0(MyError),
}

impl std::fmt::Display for MyErrorNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "my error node: {}", &self.source().expect("MyErrorNode always has a source"))
    }
}

impl std::error::Error for MyErrorNode {
   fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
       match self {
           Self::Variant0(e) => Some(e),
        }
    }
}

impl From<MyError> for MyErrorNode {
    fn from(value: MyError) -> Self {
        Self::Variant0(value)
    }
}
```

As it can be seen in the snippet above, [`hierrorchy::error_node`](macro@error_node) also implements
[`std::convert::From`]s
for each variant of the node, allowing to leverage the `?` operator in functions which return a
[`std::result::Result`].

### Complete example
```
use hierrorchy::{error_leaf,error_node};
use rand::prelude::*;
use std::error::Error;
use std::process::exit;

fn main() {
    if let Err(e) = entrypoint() {
        eprintln!("{}", e);
        exit(1);
    }
}

fn entrypoint() -> Result<(), MyErrorNode> {
    check_boolean()?;
    let value = rand::random::<i32>();
    check_value(value)?;
    Ok(())
}

fn check_boolean() -> Result<(), MyFirstErrorLeaf> {
    if rand::random() {
        Err(MyFirstErrorLeaf {})
    } else {
       Ok(())
    }
}

fn check_value(value: i32) -> Result<(), MySecondErrorLeaf> {
    if value % 2 == 0 {
       Err(MySecondErrorLeaf { value })
    } else {
        Ok(())
    }
}

#[error_leaf("first check failed")]
struct MyFirstErrorLeaf {}

#[error_leaf(format!("second check failed: value is {}", self.value))]
struct MySecondErrorLeaf {
    value: i32,
}

error_node! { type MyErrorNode<MyFirstErrorLeaf, MySecondErrorLeaf> = "error node" }
```
