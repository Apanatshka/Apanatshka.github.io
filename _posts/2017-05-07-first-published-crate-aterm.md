---
layout:   post
title:    "My first published crate: aterm"
date:     2017-05-07
category: CompSci
tags:     [annotated term, term, Rust]
---

I published my first crate to [crates.io](https://crates.io/)! It's called [`aterm`](https://crates.io/crates/aterm), and it's a library that implements the Annotated Term format. Currently it can only parse and print the normal textual format, but I'm planning to add the other three formats too at some point. There are also a number of other improvements that I have planned. But I'm going to try to not make this post a braindump of meandering thoughts. So here's what I'd like to talk about today:

* Table of Contents
{:toc}

# What are ATerms

The [<u>A</u>nnotated <u>Term</u> format](http://homepages.cwi.nl/~daybuild/daily-books/technology/aterm-guide/aterm-guide.html)[^1] originates from the [Centre for Mathematics and Computer Science (CWI)](https://www.cwi.nl/about) in Amsterdam. It describes trees and annotations on those trees. The big features are maximal sharing (subtrees that are the same are only allocated once), a compressed binary format, and a C implementation that does garbage collection. The format dates back to around 2000.

The ATerm format was used by a large set of tools in a number of research groups as an exchange format (and they probably also used the library implementations to provide the internal representation as well). Maximal sharing was super fancy, although recently research was published that this maximal sharing can in many situations be more overhead than optimisation. [^2]

For those of you who are interested in implementations of this format in other languages, have a looks at the [CWI repository](https://github.com/cwi-swat/aterms) for ATerm implementations, which includes implementations in C, Java and C#.

# Why implement this

As mentioned in the high-level description of the ATerm format, this was used by a large number of tools created by researchers as an exchange format. I actually use this format myself in my work as a PhD candidate. So I figured I could combine my wish to learn more Rust and performance engineering, with some of the knowledge I have about aging tools I use at work!

In particular, I'm currently working on an interpreter, written in Rust, that interprets Stratego Core code. [Stratego](http://strategoxt.org/) (or Stratego/XT to disambiguate from the boardgame) is a language for program transformation. The compiler can return an intermediate representation called the "core" language. This is an AST which is printed in the ATerm format. I will write about this interpreter in another blog post at a later time.

# The implementation

Ok, so here's what you get from the `aterm` crate:

1. A parser that takes in an text format ATerm string.
   - I have [a](https://gitlab.com/Apanatshka/aterm/issues/10) [number](https://gitlab.com/Apanatshka/aterm/issues/4) [of](https://gitlab.com/Apanatshka/aterm/issues/5) [issues](https://gitlab.com/Apanatshka/aterm/issues/6) open, zero-copy parsing and the other formats.
2. A printer for the text format. Again, the other formats are todo.
3. A factory trait for building ATerms.
4. An ATerm trait for matching ATerms.
5. Some basics such as the default enum for terms (`Int`, `Real`, `Application`, `List`, `Placeholder`).
6. The ATerm programming guide mentions a BLOB term, which I considered an extension point that could be typed in Rust. So you can add more stuff to the above enum with the `Blob` variant.
7. An implementation of the ATerm trait based on the above basics.
8. A reference counting implementation of the factory trait.
9. A second Rc implementation of the factory that guarantees maximal sharing.

For more details, check out the [docs](https://docs.rs/aterm). (Which I should probably extend a little :| )

# Things I learned

1. Using your own library is the only way to make sure it's actually usable! I had this implementation that compiled a while back, then I started using the factory for the parser, and nothing actually worked out. Then I started to think things over, removed a bunch type parameters, and simplified things. I had arena allocation for another factory at some point, and it was terribly broken and unsafe, so I killed that entirely. I plan on [revisiting that idea soon](https://gitlab.com/Apanatshka/aterm/issues/9) by using an arena allocation crate.
2. While simplifying and trying to get my library usable, I found out a thing I hadn't really grasped before in the Rust type system: *Associated types are unique per trait implementation*. In contrast, type parameters may vary. Therefore associated types are easier on the Rust type inference, and I solved some usability problems by using them. I still dislike how unwieldy the expressions are to add extra constraints on associated types, but I guess it's necessary...
3. Ideally I would have use higher-kinded types for this ATerm implementation. But Rust doesn't have that. So I found a way where the `ATerm` trait defines an associated type `Rec: Borrow<Self>`. That still allows people to implement the `ATerm` trait while adding in more of their own things. The basic `rc::ATerm` implementation looks like this:

```rust
struct ATerm<B>(ATermInner<Rc<ATerm<B>>, B>)
```

Where `ATermInner` is a basic implementation that contains a term and a list of annotations.

# What happened to the automata?

I wrote my last automata blog post in November last year. Since then I've been really busy. The short amounts of spare time I had left, I tried to do the benchmarking and write on the benchmarking blog post (Implementating Finite Automata Part 3), but I couldn't figure it out. Basically I'm not that familiar with performance measurement tools yet, and I was stumped by *why* my implementation was so much slower than the one I was comparing with. Since I only had an hour here or there, I couldn't effectively work on it, so I gave up for a while. When I got a bit of time again, I needed something new and exciting, not something to bash my head against. So that's why you got ATerms, and soon a post on a Stratego interpreter. I'll get back to automata afterwards. I think I can also do some performance engineering on the interpreter, so maybe I'll learn more that way and be able to finish writing about the Finite Automata. Until then, I hope you can be patient. Or just say: Eh, whatever ¯\\\_(ツ)\_/¯

[^1]: van den Brand, M. G., De Jong, H. A., Klint, P., & Olivier, P. A. (2000). Efficient annotated terms. *Software Practice and Experience*, 30(3), 259-291.

[^2]: Steindorfer, M. J., & Vinju, J. J. (2016, March). Performance Modeling of Maximal Sharing. In *Proceedings* of the 7th ACM/SPEC on *International Conference on Performance Engineering* (pp. 135-146). ACM.