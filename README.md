transmittable
=============
A tiny binary serialization library for Rust that targets `std::io` streams. It provides:

> [!WARNING]
> **This library is still in development, and its API is unstable.**

## Highlights

- Derive macro for structs and enums: `#[derive(Transmittable)]`
- Variable integer length-prefixed sequences (e.g., `Vec<T>`, `String`, etc.)
- Minimal dependencies

## Example
> [!NOTE]
> By default, the derive macro emits unsafe code for Enums.
> This behavior can be disabled by disabling the `unsafe` feature.
```rust
use transmittable::Transmittable;

#[derive(Debug, Transmittable)]
struct MyStruct {
    a: u32,
    b: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let my_struct = MyStruct {
        a: 1,
        b: "Hello World!".to_string()
    };

    let mut buf = Vec::new();
    my_struct.serialize(&mut buf)?;

    println!("Serialized struct: {:?}", buf);

    let deserialized: MyStruct = MyStruct::deserialize(&mut &buf[..])?;
    println!("Deserialized struct: {:?}", deserialized);

    Ok(())
}
```