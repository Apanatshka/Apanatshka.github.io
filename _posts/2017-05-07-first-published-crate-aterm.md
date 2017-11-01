---
layout:   post
title:    "My first published crate: aterm"
date:     2017-05-07
category: CompSci
tags:     [annotated term, term, Rust]
---

I published my first crate to [crates.io](https://crates.io/)! It's called [`aterm`](https://crates.io/crates/aterm), and it's a library that implements the <u>A</u>nnotated <u>Term</u> format. Currently it can only parse and print the normal textual format, but I'm planning to add the other three formats too at some point. There are also a number of other improvements that I have planned. But I'm going to try to not make this post a brain-dump of meandering thoughts. So here's what I'd like to discuss:

* Table of Contents
{:toc}

# What are ATerms

The Annotated Term format[^1] originates from the [Centre for Mathematics and Computer Science (CWI)](https://www.cwi.nl/about) in Amsterdam. It describes trees and annotations on those trees. The big features are maximal sharing (subtrees that are the same are only allocated once), a compressed binary format, and a C implementation that does garbage collection. The format dates back to around 2000.

```
Add(Int("1"){Value(1)}, Int("2"){Value(2)}){Value(3)}
```

(^A small tree describing addition of two numbers, with annotations on each "application" with the value)

The ATerm format was used by a number of tools in a number of research groups as an exchange format (and they probably also used the library implementations to provide the internal representation as well). Maximal sharing was super fancy, although recently research was published that this maximal sharing can in many situations be more overhead than optimisation. [^2] The compressed formats allowed pretty quick communication between different tools that sometimes explicitly held on to the [Unix philosophy](https://en.wikipedia.org/wiki/Unix_philosophy). 

For those of you who are interested in implementations of this format in other languages, have a look at the [CWI repository](https://github.com/cwi-swat/aterms), which includes implementations in C, Java and C#.

# Why implement this

As mentioned in the high-level description above, this was used by a number of tools created by researchers as an exchange format. I actually use this format myself at TU Delft. So I figured I could combine my wish to learn more Rust and performance engineering, with some of the knowledge I have about ageing tools I use!

In particular, I'm currently working on an interpreter, written in Rust, that executes Stratego Core code. [Stratego](http://strategoxt.org/) (or Stratego/XT, to disambiguate from the board game) is a language for program transformation. The Stratego compiler can return an intermediate representation called the "core" language, as an AST in the ATerm format. I'm planning to write about this interpreter in another blog post soon. (EDIT: [I have now]({% post_url 2017-08-06-a-stratego-interpreter-in-rust %}))

# The implementation

Ok, so here's what you get from the `aterm` crate:

1. A parser that takes in a text format ATerm string.
   - I have [a](https://gitlab.com/Apanatshka/aterm/issues/10) [number](https://gitlab.com/Apanatshka/aterm/issues/4) [of](https://gitlab.com/Apanatshka/aterm/issues/5) [issues](https://gitlab.com/Apanatshka/aterm/issues/6) open: basically zero-copy parsing and the other formats.
2. A printer for the text format. Again, the other formats are todo.
3. A factory trait for building ATerms.
4. An ATerm trait for matching ATerms.
5. Some basics such as the default enum for terms (`Int(i32)`, `Real(f32)`, `Application(constructor, children)`, `List(children)`, `Placeholder(placeholder_enum)`).
6. The [ATerm programming guide](http://homepages.cwi.nl/~daybuild/daily-books/technology/aterm-guide/aterm-guide.html) mentions a BLOB term, which I considered an extension point that could be typed in Rust. So you can add more stuff to the term enum with the variant `Blob(your_extension_here)`.
7. An implementation of the ATerm trait for the above basics.
8. A reference counting implementation with factory.
9. A second Rc implementation where the factory guarantees maximal sharing.

For more details, check out the [docs](https://docs.rs/aterm). (Which I should probably extend a little...)

## Highlights & Things I learned

1. Ideally I would have use higher-kinded types for this ATerm implementation. But Rust doesn't have that. So I found a way where the `ATerm` trait defines an associated type `Rec: Borrow<Self>`. That still allows people to implement the `ATerm` trait while adding in more of their own things. The basic `rc::ATerm` implementation looks like this:
  ```rust
  struct ATerm<B>(ATermInner<Rc<ATerm<B>>, B>)
  ```
  Where `ATermInner` is a basic implementation that contains a term and a list of annotations.
2. Using your own library is the only way to make sure it's actually usable! I had this implementation that compiled a while back, then I started using the factory for the parser, and nothing actually worked out. Then I started to think things over, removed a bunch type parameters, generally simplified things. I'm now a bit unhappy with the allocation characteristics, but the crate isn't 1.0 yet, I can still tinker with it a little although I don't want to introduce big breaking changes anymore. 
3. I had an arena allocation implementation at some point. It was terribly broken, so I had to cut it out. I plan on [revisiting that idea soon](https://gitlab.com/Apanatshka/aterm/issues/9) by using someone else's arena allocation crate.
4. While simplifying and trying to get my library usable, I found out a thing I hadn't really grasped before in the Rust type system: *Associated types are unique per trait implementation*. So for my `ATerm` trait I made the `Rec` and `Blob` types associative, and that made it much more usable. You see, associated types are easier on Rust's type inference: Given a fully concrete type there is definitely only one trait implementation possible. And it's not as limiting for the implementor as you might think. You can still use type parameters in your `impl`s:
  ```rust
impl<Rec, B> ATerm for ATermInner<Rec, B>
       where Rec: Borrow<ATermInner<Rec, B>>
{
       type Rec = Rec;
       type Blob = B;

       #[inline]
       fn into_inner(self) -> ATermInner<Rec, B> {
           self
       }
       #[inline]
       fn as_inner(&self) -> &ATermInner<Self::Rec, Self::Blob> {
           self
       }
}
```
  The only thing I dislike about associated types is how large your where clauses get when you want to add extra constraints. While working on my Stratego interpreter, at some point I had where clauses like this:
  ```rust
where F: 'a + ATermFactory<'a, B, ATermRef = A>,
         A: Borrow<<F as ATermFactory<'a, B>>::ATerm> + Clone,
         <F as ATermFactory<'a, B>>::ATerm: Eq,
         B: 'a
```
  I was already using type parameter `A` so I didn't have to use `<F as ATermFactory<'a, B>>::ATermRef`. I found a slight improvement when I defined type aliases for `ATermRef` and `ATerm` from a factory. Later I just defined the concrete types I was using with some type aliases (for easy changing later) and went with that. Makes it slightly harder to change them later if I start to depend on concrete parts of the types, but most code was already written in a generic way, so it shouldn't be too much of a pain. It's kind of sad that writing generic code is so ugly and sometimes impossible, I like writing code where I only require a minimal contract on what the types need to be capable of. 

# What happened to automata?

I wrote my last automata blog post in November last year. Since then I've been really busy. In the short amounts of spare time I had left, at first I tried to do the benchmarking for the third Finite Automata post. But I couldn't figure out how to do it right, or maybe it was all in reading the trace output. Basically I'm not that familiar with performance measurement tools yet, and I was mostly  stumped by *why* my implementation was so much slower than the one I was comparing with (I wasn't surprised *that* it was slower, I was comparing to [BurntSushi's `aho-corasick`](https://crates.io/crates/aho-corasick), good luck beating his stuff ^^). Since I only had an hour here or there, I couldn't effectively work on it, so I gave up for a while. When I got a bit of time again, I needed something new and exciting, not something to bash my head against. So that's why you got ATerms, and soon a post on my Stratego interpreter. I'll get back to automata afterwards. I think I can also do some performance engineering on the interpreter, so maybe I'll learn more that way and use that knowledge to finish writing about the Finite Automata. Until then, I hope you can be patient. Or just say: Eh, whatever ¯\\\_(ツ)\_/¯

[^1]: van den Brand, M. G., De Jong, H. A., Klint, P., & Olivier, P. A. (2000). Efficient annotated terms. *Software Practice and Experience*, 30(3), 259-291.

[^2]: Steindorfer, M. J., & Vinju, J. J. (2016, March). Performance Modeling of Maximal Sharing. In *Proceedings* of the 7th ACM/SPEC on *International Conference on Performance Engineering* (pp. 135-146). ACM.
