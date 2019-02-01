# Snax: JSX-ish for Rust
**Early WIP**

Templates should just be Rust code. This is the philosophy behind JSX and React and I think it's great.

[typed-html](https://github.com/bodil/typed-html) is the closest thing to what I want. It's a bit rough around the edges and I had technical problems trying to work on it, so I decided to see how far I could get with a clean slate.

## Requirements
* Rust 1.32+. State of the art, yo.

## Goals
* JSX-like syntax via proc macro
	* Embed arbitrary Rust code in braces
* Output to (typed?) virtual DOM, have builtin mechanism to turn DOM into string
* (Maybe) compile-time template checking. This isn't important to me right now
* Fast Enough™

## Example
```rust
use snax::{snax, Fragmment, HtmlContent};

fn main() {
	println!("{}", render_page());
}

fn render_page() -> HtmlContent {
	snax!(
		<html>
			<head>
				<title>"Hello, Snax!"</title>
			</head>
			<body>
				{ Fragment::from((0..16).map(render_age)) }
			</body>
		</html>
	)
}

fn render_age(age: u32) -> HtmlContent {
	snax!(
		<div class="age-widget">
			{ format!("Age: {}", age) }
		</div>
	)
}
```

## License
Snax is available under the MIT license. See [LICENSE.txt](LICENSE.txt) for details.