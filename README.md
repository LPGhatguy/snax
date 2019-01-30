# Snax: JSX-ish for Rust
**Super super early WIP**

Templates should just be Rust code. This is the philosophy behind JSX and React and I think it's great.

[typed-html](https://github.com/bodil/typed-html) is the closest thing to what I want. It's a big rough around the edges and I had technical problems trying to work on it, so I decided to see how far I could get with a clean slate.

## Goals
* JSX-like syntax via proc macro
	* Embed arbitrary Rust code in braces
* Output to (typed?) virtual DOM, have builtin mechanism to turn DOM into string
* (Maybe) compile-time template checking. This isn't important to me right now
* Fast Enough™