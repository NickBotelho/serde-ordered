use serde::{Deserialize, Serialize};
use serde_ordered::DeserializeOrdered;

#[derive(Deserialize, Serialize, Default)]
struct Foo {
    pub buz: i32,
    pub biz: Option<String>,
    pub bar: Bar,
    pub bop: u64
}

#[derive(Deserialize, Serialize, Default)]
struct Bar {
    pub buf: i32,
    pub bif: String
}

#[derive(DeserializeOrdered)]
struct SlimFoo {
    #[serde(order=1)]
    pub biz: Option<String>,

    #[serde(order=2)]
    pub bar: SlimBar,
}

#[derive(DeserializeOrdered)]
struct SlimBar {
    #[serde(order=1)]
    pub bif: String
}

#[test]
fn deserialize_json(){
    let foo_str = r#"[1, null, [100, "100"]]"#;
    let slim_foo = serde_json::from_str::<SlimFoo>(&foo_str).unwrap();

    assert!(slim_foo.biz.is_none());
    assert_eq!(slim_foo.bar.bif, "100");
}

#[test]
fn deserialize_messagepack(){
    let mut foo = Foo::default();
    foo.biz = Some("biz".to_string());
    foo.bar.bif = "99".into();

    let foo_bytes = rmp_serde::to_vec(&foo).unwrap();

    let slim_foo = rmp_serde::from_slice::<SlimFoo>(&foo_bytes).unwrap();

    assert_eq!(slim_foo.biz.unwrap(), "biz");
    assert_eq!(slim_foo.bar.bif, "99");
}