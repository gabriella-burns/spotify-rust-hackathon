# Rust Errors Teaching Guide

This guide explains the 5 intentional compiler errors in `main.rs` that demonstrate key Rust concepts.

## 🎯 **Learning Objectives**

After studying these errors, you should understand:
- **Ownership Rules**: Who owns data and when it can be borrowed
- **Lifetime Annotations**: How to specify how long references are valid
- **Borrowing Rules**: When you can have multiple references to data
- **Trait Implementations**: How to correctly implement traits
- **Move Semantics**: What happens when data is moved vs borrowed

---

## ❌ **Error #1: Ownership Issue - Dangling Reference**

### **Error Location**: Lines 514-517
```rust
fn ownership_error_example() -> &SpotifyConfig {
    let config = SpotifyConfig::from_env().unwrap();
    &config  // ERROR: Can't return reference to local variable
}
```

### **Compiler Error**:
```
error[E0515]: cannot return reference to local variable `config`
   --> src/main.rs:516:5
    |
516 |     &config // ERROR: Can't return reference to local variable
    |     ^^^^^^^ returns a reference to data owned by the current function
```

### **What's Happening**:
- The function creates a local variable `config` (it owns this data)
- The function tries to return a reference `&config` to this local variable
- When the function ends, `config` is destroyed (dropped)
- The returned reference would point to destroyed memory (dangling reference)

### **Rust Concept**: Ownership and Borrowing
- **Ownership**: Each value has exactly one owner
- **Borrowing**: You can borrow data temporarily, but the owner must outlive the borrow
- **Dangling References**: References that point to destroyed data are forbidden

### **How to Fix**:
```rust
// Option 1: Return owned value (transfer ownership)
fn ownership_error_example() -> SpotifyConfig {
    SpotifyConfig::from_env().unwrap()
}

// Option 2: Take a reference as parameter (borrow from caller)
fn ownership_error_example(config: &SpotifyConfig) -> &SpotifyConfig {
    config
}
```

---

## ❌ **Error #2: Lifetime Issue - Missing Lifetime Annotation**

### **Error Location**: Lines 524-526
```rust
fn lifetime_error_example(tracks: &[Track]) -> &Track {
    &tracks[0]  // ERROR: Missing lifetime annotation
}
```

### **Compiler Error**:
```
error[E0106]: missing lifetime specifier
   --> src/main.rs:524:33
    |
524 | fn lifetime_error_example(tracks: &[Track]) -> &Track {
    |                                 ^ expected named lifetime parameter
```

### **What's Happening**:
- The function takes a reference `&[Track]` as input
- It returns a reference `&Track` from that input
- Rust needs to know how long the returned reference is valid
- Without lifetime annotation, Rust can't ensure memory safety

### **Rust Concept**: Lifetimes
- **Lifetimes**: Specify how long references are valid
- **Lifetime Parameters**: Named lifetimes (like `'a`) that connect input and output references
- **Memory Safety**: Ensures references don't outlive the data they point to

### **How to Fix**:
```rust
// Add lifetime parameter to connect input and output references
fn lifetime_error_example<'a>(tracks: &'a [Track]) -> &'a Track {
    &tracks[0]
}

// Or use elision rules (Rust can infer this simple case)
fn lifetime_error_example(tracks: &[Track]) -> &Track {
    &tracks[0]
}
```

---

## ❌ **Error #3: Borrowing Violation - Multiple Mutable Borrows**

### **Error Location**: Lines 533-541
```rust
fn borrowing_error_example(tracks: &mut Vec<Track>) {
    let first = &mut tracks[0];  // First mutable borrow
    let second = &mut tracks[1]; // ERROR: Second mutable borrow while first is still active
    println!("{} and {}", first.name, second.name);
}
```

### **Compiler Error**:
```
error[E0499]: cannot borrow `*tracks` as mutable more than once at a time
   --> src/main.rs:539:23
    |
539 |     let second = &mut tracks[1]; // ERROR: Second mutable borrow while first is still active
    |                       ^^^^^^ second mutable borrow occurs here
```

### **What's Happening**:
- The function has a mutable reference `&mut Vec<Track>`
- It creates a mutable borrow `&mut tracks[0]` and stores it in `first`
- While `first` is still active, it tries to create another mutable borrow `&mut tracks[1]`
- Rust's borrowing rules prevent multiple mutable references to the same data

### **Rust Concept**: Borrowing Rules
- **At any given time, you can have either**:
  - One mutable reference (`&mut T`)
  - Any number of immutable references (`&T`)
- **But not both at the same time**
- **References must always be valid** (no dangling references)

### **How to Fix**:
```rust
// Option 1: Use the references sequentially
fn borrowing_error_example(tracks: &mut Vec<Track>) {
    let first = &mut tracks[0];
    println!("First: {}", first.name);
    // first goes out of scope here
    
    let second = &mut tracks[1];
    println!("Second: {}", second.name);
}

// Option 2: Use split_at_mut for non-overlapping slices
fn borrowing_error_example(tracks: &mut Vec<Track>) {
    let (first_slice, second_slice) = tracks.split_at_mut(1);
    let first = &mut first_slice[0];
    let second = &mut second_slice[0];
    println!("{} and {}", first.name, second.name);
}
```

---

## ❌ **Error #4: Trait Implementation Error - Wrong Method Signature**

### **Error Location**: Lines 550-554
```rust
impl Formattable for String {
    fn format(self) -> String {  // ERROR: Should be `&self`, not `self`
        self
    }
}
```

### **Compiler Error**:
```
error[E0053]: method `format` has an incompatible type for trait
   --> src/main.rs:552:15
    |
552 |     fn format(self) -> String {
    |               ^^^^ expected `&std::string::String`, found `std::string::String`
```

### **What's Happening**:
- The `Formattable` trait defines `fn format(&self) -> String`
- The implementation tries to use `fn format(self) -> String`
- `&self` means "borrow self" (immutable reference)
- `self` means "take ownership of self" (moves the value)
- These are different method signatures

### **Rust Concept**: Trait Implementations
- **Trait Methods**: Must match the exact signature defined in the trait
- **Method Receivers**: 
  - `&self`: Borrows the value (immutable reference)
  - `&mut self`: Borrows the value mutably
  - `self`: Takes ownership (moves the value)
- **Trait Coherence**: Implementations must be consistent with trait definitions

### **How to Fix**:
```rust
impl Formattable for String {
    fn format(&self) -> String {  // Fixed: use &self to match trait
        self.clone()  // Clone since we can't move self
    }
}
```

---

## ❌ **Error #5: Iterator Misuse - Using Moved Value**

### **Error Location**: Lines 560-575
```rust
fn iterator_error_example() {
    let tracks = vec![
        Track {
            name: "Song 1".to_string(),
            artists: vec![Artist { name: "Artist 1".to_string() }],
        }
    ];
    
    let iterator = tracks.into_iter();  // Moves ownership of tracks
    println!("First track: {:?}", tracks);  // ERROR: tracks was moved
}
```

### **Compiler Error**:
```
error[E0382]: borrow of moved value: `tracks`
   --> src/main.rs:574:35
    |
574 |     println!("First track: {:?}", tracks); // ERROR: tracks was moved
    |                                   ^^^^^^ value borrowed here after move
```

### **What's Happening**:
- `tracks` is a `Vec<Track>` (owned data)
- `into_iter()` takes ownership of `tracks` and moves it into the iterator
- After the move, `tracks` is no longer valid
- Trying to use `tracks` after the move causes a compiler error

### **Rust Concept**: Move Semantics
- **Move**: When ownership is transferred, the original variable becomes invalid
- **Copy**: Some types (like `i32`) implement `Copy` and are copied instead of moved
- **Clone**: Explicit copying that creates a new owned value
- **Borrowing**: Using references to avoid moving data

### **How to Fix**:
```rust
// Option 1: Use iter() instead of into_iter() (borrows instead of moves)
fn iterator_error_example() {
    let tracks = vec![
        Track {
            name: "Song 1".to_string(),
            artists: vec![Artist { name: "Artist 1".to_string() }],
        }
    ];
    
    let iterator = tracks.iter();  // Borrows tracks, doesn't move
    println!("First track: {:?}", tracks);  // Works!
}

// Option 2: Clone the data before moving
fn iterator_error_example() {
    let tracks = vec![
        Track {
            name: "Song 1".to_string(),
            artists: vec![Artist { name: "Artist 1".to_string() }],
        }
    ];
    
    let iterator = tracks.clone().into_iter();  // Clone first, then move
    println!("First track: {:?}", tracks);  // Works!
}
```

---

## 🎓 **Teaching Tips**

### **For Instructors**:
1. **Start with Error #1**: Ownership is the foundation of Rust
2. **Use Visual Aids**: Draw boxes and arrows to show ownership flow
3. **Explain the "Why"**: Don't just show the fix, explain why Rust prevents these errors
4. **Show Real-World Impact**: These errors prevent common bugs in other languages

### **For Students**:
1. **Read the Error Messages**: Rust's error messages are very helpful
2. **Understand the Concepts**: Don't just memorize fixes
3. **Practice**: Try creating similar errors and fixing them
4. **Use the Compiler**: Let Rust guide you to correct code

### **Common Patterns**:
- **Ownership Transfer**: Use `into_iter()`, `clone()`, or `to_owned()`
- **Borrowing**: Use `&` for immutable, `&mut` for mutable references
- **Lifetimes**: Use `'a` to connect input and output references
- **Traits**: Match the exact method signature from the trait definition

---

## 🔧 **Quick Reference**

| Error Type | Rust Concept | Common Fix |
|------------|--------------|------------|
| Dangling Reference | Ownership | Return owned value or borrow from parameter |
| Missing Lifetime | Lifetimes | Add lifetime parameter or use elision |
| Multiple Mutable Borrows | Borrowing Rules | Use sequentially or `split_at_mut()` |
| Wrong Trait Implementation | Traits | Match exact method signature |
| Using Moved Value | Move Semantics | Use `iter()` instead of `into_iter()` or clone |

---

## 🚀 **Next Steps**

After understanding these errors:
1. **Practice**: Create your own examples of each error type
2. **Explore**: Learn about more advanced Rust concepts
3. **Build**: Apply these concepts to real projects
4. **Teach**: Help others understand Rust's safety guarantees

Remember: **Rust's compiler is your friend!** It catches these errors at compile time, preventing runtime bugs that are common in other languages. 
