transmittable
=============
A tiny binary serialization library for Rust that targets `std::io` streams. It provides:

- A `Transmittable` trait with `serialize` and `deserialize` methods over `Write`/`Read`
- A derive macro to implement the trait for your structs and enums
- A straightforward, length-prefixed wire format with big-endian integers by default

## Highlights

- Derive for structs and enums: `#[derive(Transmittable)]`
- Length-prefixed sequences (e.g., `Vec<T>`, `String`, etc.)
- Minimal dependencies

## Usage
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