# Snax: JSX-like for Rust
<!-- To regenerate this README, use `cargo readme -r snax > README.md` -->

Snax is an implementation of a JSX-like grammar for Rust. You can use it as
part of the `snax` crate, which provides a complete, untyped DOM that's
quick to turn into a string.

If you're a proc macro author, you can also consume the `snax_syntax` to
produce whatever structures you need for your project. This can be used to
support a typed DOM or any React-like framework!

### Installation
Snax requires Rust 1.32 or newer.

It isn't published to crates.io yet, but you can use a Git dependency:

```toml
snax = { git = "https://github.com/LPGhatguy/snax.git" }
```

Some things are still a bit in flux, so I'm sorry in advance if I break
anything!

### Examples

#### Simple Page
```rust
use snax::snax;

fn main() {
    let page_title = "Hello, world, from Snax!";

    let page = snax!(
        /* Snax supports regular multi-line Rust comments. */
        <html>
            <head>
                /*
                    Literal strings need to be quoted, unlike JSX.
                    This makes whitespace much more explicit, which is useful!
                */
                <title>"Hello, Snax!"</title>
            </head>
            <body>
                <h1>
                    /*
                        Snax supports embedding arbitrary Rust expressions as long as they
                        can be turned into a DOM node. String and &str work great here!
                    */
                    { page_title }
                </h1>
            </body>
        </html>
    );

    // The result of the snax! macro is an HtmlContent.
    // It implements Display and gives you compact HTML without a doctype!
    println!("<!doctype html>");
    println!("{}", page);
}
```

### License
Snax is available under the MIT license. See [LICENSE.txt](LICENSE.txt) for
details.
