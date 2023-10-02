//! Note the documentation for the primitives **str** and **[T]** (also
//! called 'slice'). Many method calls on **String** and **Vec<T>** are actually
//! calls to methods on **str** and **[T]** respectively, via **deref
//! coercions**.



///doc comment
/*
  This is a block comment describing a complex function.
*/
fn main(x : Field, y : pub Field) {
    assert(x != y);
}

///qwe
///rty
fn simple_fn(){}

///qwe
///rty
pub fn pub_fn(){}

//! Data available to helper can be found in [Helper](struct.Helper.html). And there are more
//! examples in [HelperDef](trait.HelperDef.html) page.
//!
//! You can learn more about helpers by looking into source code of built-in helpers.
//!
//! Like our JavaScript counterparts, handlebars allows user to define simple helpers with
//! a scripting language, [rhai](https://docs.rs/crate/rhai/). This can be enabled by
//! turning on `script_helper` feature flag.

/// struct
struct MyStruct {
  mana: i32,
  name: String,
}

///a 
///lot 
///of 
/**
doc 
comments
*/
fn another_fn(){}