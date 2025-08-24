# `serde-ordered`

A procedural macro for deserializing un-keyed, ordered arrays into keyed structs using Serde.

## Features

- Deserialize ordered arrays into structs with named fields.
- Supports optional fields and nested structures.
- Works with JSON, MessagePack, and other Serde-compatible formats.

## Motivation
Working with large, un-keyed array structures can be inconvenient. If a struct has upwards of 50 fields that include large nested structs, making sure every field is present can be tedious and slow down development time. Often times, typing out all 50 fields in not warranted, especially if the plan is to only leverage a small subset of those fields. serde-ordered lets you create a slimmed down version of the parent class, placing orders on the fields of interest

Here we have a parent struct `Foo`
```rust
#[derive(Deserialize, Serialize)]
struct Foo {
    pub buz: i32,
    pub biz: Option<String>,
    pub bar: Bar,
    pub bop: u64
}

#[derive(Deserialize, Serialize)]
struct Bar {
    pub buf: i32,
    pub bif: String
}
```

If we tried to deserialize Foo into a slimmed down struct like `SlimFoo`
```rust
#[derive(Deserialize, Serialize)]
struct SlimFoo {
    pub biz: Option<String>,
    pub bar: SlimBar,
}

#[derive(Deserialize, Serialize)]
struct SlimBar {
    pub bif: String
}
```
against an un-keyed MessagePack message like `[1, null, [100, "100"], 1]` it would error due to a length mismatch, requring developers to ensure they have typed out every field. This could also introduce problems if an upstream provider changed the struct without notifying the consumer which would cause a length mismatch error, Hense `serde-ordered`

## Installation

Add this to your `Cargo.toml`: the proc macro within serde-ordered
```toml
[dependencies]
serde-ordered = "*"
```

## Usage
Simply derive the `DeserializeOrdered` trait on the struct and tag each field of interest with an index/order

```rust
#[derive(DeserializeOrdered)]
struct SlimFoo {
    #[order(0)]
    pub buz: i32,

    #[order(2)]
    pub bar: SlimBar,
}

#[derive(DeserializeOrdered)]
struct SlimBar {
    #[order(1)]
    pub bif: String
}
```
This will automatically implement a custom deserializer that only attempts to deserialize the fields on the specified orders. 

## License

MIT