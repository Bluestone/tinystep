# TinyStep #

TinyStep is a small wrapper library for interacting with a [SmallStep][ss]
instance. This can allow you to issue certificates, or interact with the rest
of the SmallStep Certificate authority from your rust code.

***This is currently a heavy work in progress, as an attempt to implement
the parts we need first, but ideally we create a full HTTP Client
implementation too.***

To start using tinystep you can put the following in your Cargo.toml:

```toml
[dependencies]
tinystep = { git = "https://github.com/Bluestone/tinystep" }
```

## How to Use ##

Currently the best place to look for examples on how to use tinystep, are
examples listed inside of the `examples/` folder. Once the first version of
this library is published, we will publish a link to the documentation here
too, but in the meantime the examples are the best place to look.

## Supported Rust Versions ##

Since TinyStep uses a particular http client: [isahc][isahc] we only support
the latest version of Rust at this time. We plan to extend this as time goes
on.

[ss]: https://smallstep.com/
[isahc]: https://github.com/sagebind/isahc/